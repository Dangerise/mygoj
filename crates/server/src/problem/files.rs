use super::*;

pub async fn can_access_problem_file(
    user: Option<&LoginedUser>,
    pid: &Pid,
    path: &str,
) -> Result<bool, ServerError> {
    let problem = get_problem(pid).await?;
    let file = problem
        .files
        .iter()
        .find(|d| d.path == path)
        .ok_or(ServerError::NotFound)?;
    let mut ret = file.is_public;
    if let Some(user) = user {
        ret |= user.privilege.edit_problems;
        ret |= Some(user.uid) == problem.owner;
    }
    Ok(ret)
}

pub async fn clean_unused_problem_files(pid: &Pid) -> Result<u64, ServerError> {
    let lock = problem_write_lock(pid).await;
    let files = get_problem(pid).await?.files.clone();
    let mut joinset = tokio::task::JoinSet::new();
    let storage = problem_storage_path(pid);
    let mut stream = fs::read_dir(&storage)
        .await
        .map_err(ServerError::into_internal)?;
    while let Some(entry) = stream
        .next_entry()
        .await
        .map_err(ServerError::into_internal)?
    {
        let filename = entry.file_name();
        let should_clean = 'tag: {
            let Ok(filename) = str::from_utf8(filename.as_encoded_bytes()) else {
                break 'tag true;
            };
            let Ok(uuid) = filename.parse::<Uuid>() else {
                break 'tag true;
            };
            !files.iter().any(|d| d.uuid == uuid)
        };
        if should_clean {
            tracing::debug!("clean {}", filename.display());
            joinset.spawn(fs::remove_file(storage.join(filename)));
        }
    }
    let mut count = 0;
    while let Some(ret) = joinset.join_next().await {
        let _ = ret.map_err(ServerError::into_internal)?;
        count += 1;
    }
    drop(lock);
    Ok(count)
}

use axum::extract::{Extension, Multipart, Path};
use shared::user::LoginedUser;
pub async fn commit_problem_files(
    Extension(login): Extension<Option<LoginedUser>>,
    Path(pid): Path<Pid>,
    mut multipart: Multipart,
) -> Result<(), ServerError> {
    let login = login.fuck()?;
    tracing::trace!("got");
    if !can_manage_problem(&login, &pid).await? {
        return Err(ServerError::Fuck);
    }
    let mut problem_files = get_problem(&pid)
        .await?
        .files
        .iter()
        .map(Clone::clone)
        .map(|x| (x.path.clone(), x))
        .collect::<HashMap<_, _>>();

    let storage = problem_storage_path(&pid);
    let mut meta = None;
    let mut upload = Vec::new();
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ServerError::Network)?
    {
        let name = field.name().fuck()?;
        if name == "meta" {
            let json = field.text().await.fuck()?;
            let value: FileChangeMeta = serde_json::from_str(&json).fuck()?;
            meta = Some(value);
        } else if name == "file" {
            let index: usize = field.file_name().fuck()?.parse().fuck()?;
            let uuid = Uuid::new_v4();
            let path = storage.join(uuid.to_string());
            tracing::trace!("writing {} to {} size", index, uuid);
            let mut size = 0;
            let file = tokio::fs::File::create_new(path)
                .await
                .map_err(ServerError::into_internal)?;
            let mut writer = tokio::io::BufWriter::new(file);
            while let Some(chunk) = field.chunk().await.map_err(ServerError::into_internal)? {
                size += chunk.len();
                writer
                    .write_all(&chunk)
                    .await
                    .map_err(ServerError::into_internal)?;
            }
            tracing::trace!("written size {}", size);
            writer.flush().await.map_err(ServerError::into_internal)?;
            upload.push((index, uuid));
        } else {
            return Err(ServerError::Fuck);
        }
    }

    upload.sort_unstable_by_key(|x| x.0);

    if upload.iter().enumerate().any(|(x, y)| x != y.0) {
        return Err(ServerError::Fuck);
    }

    let meta = meta.fuck()?;

    let lock = problem_write_lock(&pid).await;
    let mut upload = upload.iter();
    let mut to_remove = 0;

    for evt in meta.evts {
        use FileChangeEvent::*;
        match evt {
            SetPriv(path) => {
                let f = problem_files.get_mut(&path).fuck()?;
                if !f.is_public {
                    return Err(ServerError::Fuck);
                }
                f.is_public = false;
            }
            SetPub(path) => {
                let f = problem_files.get_mut(&path).fuck()?;
                if f.is_public {
                    return Err(ServerError::Fuck);
                }
                f.is_public = true;
            }
            Remove(path) => {
                let _ = problem_files.remove(&path).fuck()?;
                to_remove += 1;
            }
            Upload { path, time, size } => {
                let (_, uuid) = *upload.next().fuck()?;
                if problem_files.contains_key(&path) {
                    let file = problem_files.get_mut(&path).unwrap();
                    file.last_modified = time;
                    file.size = size;
                    file.uuid = uuid;
                    to_remove += 1;
                } else {
                    problem_files.insert(
                        path.clone(),
                        ProblemFile {
                            path: path.clone(),
                            uuid,
                            is_public: false,
                            size,
                            last_modified: time,
                        },
                    );
                }
            }
        }
    }

    tokio::spawn(async move {
        let ret = async {
            let problem_files = Arc::new(problem_files.into_iter().map(|x| x.1).collect());

            tracing::trace!("old {:#?}", &get_problem(&pid).await.unwrap().files);
            tracing::trace!("new {:#?}", &problem_files);

            let mut problem = get_problem(&pid).await?.as_ref().clone();
            problem.files = problem_files;
            set_problem(&pid, Arc::new(problem)).await?;
            drop(lock);
            Ok::<_, ServerError>(())
        }
        .await;
        let count = clean_unused_problem_files(&pid).await?;
        if ret.is_ok() && count != to_remove {
            return Err(ServerError::Internal("clean files wrong".into()));
        }
        Ok(())
    })
    .await
    .unwrap()?;

    Ok(())
}

use shared::download::DownloadToken;

#[derive(Debug)]
struct DownloadTokenInfo {
    pid: Pid,
    path: CompactString,
}

#[dynamic]
static DOWNLOAD_TOKENS: DashMap<DownloadToken, DownloadTokenInfo> = DashMap::new();

use axum::extract::Query;
use axum::response::IntoResponse;
use axum_extra::response::FileStream;
use tokio_util::io::ReaderStream;

pub async fn require_problem_file_download_token(
    login: Option<&LoginedUser>,
    pid: Pid,
    path: CompactString,
) -> Result<DownloadToken, ServerError> {
    if can_access_problem_file(login, &pid, &path).await? {
        let token = DownloadToken::new();
        let info = DownloadTokenInfo { pid, path };
        DOWNLOAD_TOKENS.insert(token, info);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_mins(1)).await;
            DOWNLOAD_TOKENS.remove(&token);
        });
        Ok(token)
    } else {
        Err(ServerError::NoPrivilege)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DownloadQuery {
    token: Option<DownloadToken>,
}

pub async fn file_download(
    Extension(login): Extension<Option<LoginedUser>>,
    Path((pid, path)): Path<(Pid, CompactString)>,
    Query(DownloadQuery { token }): Query<DownloadQuery>,
) -> Result<impl IntoResponse, ServerError> {
    if let Some(token) = token
        && let Some((_, info)) = DOWNLOAD_TOKENS.remove(&token)
    {
        if info.path != path || info.pid != pid {
            return Err(ServerError::Fuck);
        }
    } else if !can_access_problem_file(login.as_ref(), &pid, &path).await? {
        return Err(ServerError::NoPrivilege);
    };
    let file = get_problem_file(&pid, &path).await?;
    let file = fs::File::open(file)
        .await
        .map_err(ServerError::into_internal)?;
    let stream = ReaderStream::new(file);
    let stream = FileStream::new(stream).file_name(path);
    Ok(stream)
}

pub async fn get_problem_file_meta(
    user: Option<&LoginedUser>,
    pid: &Pid,
    path: &str,
) -> Result<ProblemFile, ServerError> {
    let problem = get_problem(&pid).await?;
    let mat = problem
        .files
        .iter()
        .find(|d| d.path == path)
        .ok_or(ServerError::Fuck)?;
    if !can_access_problem_file(user, pid, path).await? {
        return Err(ServerError::NoPrivilege);
    }
    Ok(mat.clone())
}
