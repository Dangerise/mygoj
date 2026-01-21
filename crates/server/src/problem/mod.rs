mod cache;
mod db;

use super::ServerError;
use compact_str::CompactString;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
pub use shared::problem::*;
use shared::user::Uid;
use static_init::dynamic;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};
use tokio_util::io::ReaderStream;

#[dynamic]
static PROBLEM_LOCKS: DashMap<Pid, Arc<RwLock<()>>> = DashMap::new();

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Problem {
    pub pid: Pid,
    pub owner: Option<Uid>,
    pub title: CompactString,
    pub statement: Arc<String>,
    pub memory_limit: u32,
    pub time_limit: u32,
    pub testcases: Arc<Vec<Testcase>>,
    pub files: Arc<Vec<ProblemFile>>,
}

pub async fn get_problem(pid: &Pid) -> Result<Arc<Problem>, ServerError> {
    if let Some(ret) = cache::get_problem(pid).await {
        return Ok(ret);
    }
    let ret = db::get_problem(pid).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => ServerError::NotFound,
        _ => ServerError::into_internal(e),
    })?;
    cache::update_problem(pid, ret.clone()).await;
    Ok(ret)
}

pub async fn set_problem(pid: &Pid, data: Arc<Problem>) -> Result<(), ServerError> {
    db::set_problem(pid, &data)
        .await
        .map_err(ServerError::into_internal)?;
    cache::update_problem(pid, data).await;
    Ok(())
}

pub async fn get_problem_editable(pid: &Pid) -> Result<ProblemEditable, ServerError> {
    let problem = get_problem_front(pid).await?;
    Ok(ProblemEditable {
        owner: problem.owner,
        time_limit: problem.time_limit,
        memory_limit: problem.memory_limit,
        statement: problem.statement,
        title: problem.title,
    })
}

pub async fn get_problem_front(pid: &Pid) -> Result<ProblemFront, ServerError> {
    let problem = get_problem(pid).await?;

    let front = ProblemFront {
        title: problem.title.clone(),
        statement: (*problem.statement).clone(),
        time_limit: problem.time_limit,
        memory_limit: problem.memory_limit,
        owner: problem.owner,
        public_files: problem
            .files
            .iter()
            .filter(|f| f.is_public)
            .map(|f| f.path.clone())
            .collect(),
        pid: pid.clone(),
    };

    Ok(front)
}

#[must_use]
pub async fn problem_read_lock(pid: &Pid) -> OwnedRwLockReadGuard<()> {
    PROBLEM_LOCKS
        .entry(pid.clone())
        .or_default()
        .clone()
        .read_owned()
        .await
}

#[must_use]
pub async fn problem_write_lock(pid: &Pid) -> OwnedRwLockWriteGuard<()> {
    PROBLEM_LOCKS
        .entry(pid.clone())
        .or_default()
        .clone()
        .write_owned()
        .await
}

pub async fn problem_data(pid: Pid) -> Result<ProblemData, ServerError> {
    let problem = get_problem(&pid).await?;
    let data = ProblemData {
        pid,
        files: (*problem.files).clone(),
        testcases: (*problem.testcases).clone(),
        time_limit: problem.time_limit,
        memory_limit: problem.memory_limit,
    };
    Ok(data)
}

fn problem_file_path(pid: &Pid, path: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join("mygoj")
        .join("problems")
        .join(&pid.0)
        .join(path)
}

use axum::body::Body;
use axum::response::Response;

pub async fn send_problem_file(pid: Pid, filename: &str) -> Result<Response, ServerError> {
    let path = problem_file_path(&pid, filename);
    if !path.exists() {
        return Err(ServerError::NotFound);
    }
    let file = fs::File::open(path)
        .await
        .map_err(ServerError::into_internal)?;
    let stream = ReaderStream::with_capacity(file, 1 << 20);
    let resp = Response::new(Body::from_stream(stream));
    Ok(resp)
}

pub async fn can_manage_problem(user: &LoginedUser, pid: &Pid) -> Result<bool, ServerError> {
    if user.privilege.edit_problems {
        return Ok(true);
    }
    let p = get_problem(pid).await?;
    if p.owner == Some(user.uid) {
        return Ok(true);
    }
    return Ok(false);
}

use axum::extract::{Extension, Multipart, Path};
use shared::user::LoginedUser;
pub async fn commit_problem_files(
    Extension(login): Extension<Option<LoginedUser>>,
    Path(pid): Path<Pid>,
    mut multipart: Multipart,
) -> Result<(), ServerError> {
    let login = login.ok_or(ServerError::Fuck)?;
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

    let mut meta = None;
    let mut upload = Vec::new();
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ServerError::Network)?
    {
        let name = field.name().ok_or(ServerError::Fuck)?;
        if name == "meta" {
            let json = field.text().await.map_err(|_| ServerError::Fuck)?;
            let value: FileChangeMeta =
                serde_json::from_str(&json).map_err(|_| ServerError::Fuck)?;
            meta = Some(value);
        } else if name == "file" {
            let index: usize = field
                .file_name()
                .ok_or(ServerError::Fuck)?
                .parse()
                .map_err(|_| ServerError::Fuck)?;
            let (file, path) = tokio::task::spawn_blocking(|| {
                let t = tempfile::NamedTempFile::new()
                    .map_err(ServerError::into_internal)?
                    .into_parts();
                Ok(t)
            })
            .await
            .unwrap()?;

            tracing::trace!("writing {} to {}", index, path.display());

            let file = tokio::fs::File::from_std(file);
            let mut writer = tokio::io::BufWriter::new(file);
            while let Some(chunk) = field.chunk().await.map_err(ServerError::into_internal)? {
                writer
                    .write_all(&chunk)
                    .await
                    .map_err(ServerError::into_internal)?;
            }
            upload.push((index, path));
        } else {
            return Err(ServerError::Fuck);
        }
    }

    upload.sort_unstable_by_key(|x| x.0);

    if upload.iter().enumerate().any(|(x, y)| x != y.0) {
        return Err(ServerError::Fuck);
    }

    let meta = meta.ok_or(ServerError::Fuck)?;

    tokio::spawn(async move {
        let lock = problem_write_lock(&pid).await;
        let mut upload = upload.into_iter();
        for evt in meta.evts {
            use FileChangeEvent::*;
            match evt {
                SetPriv(path) => {
                    let f = problem_files.get_mut(&path).ok_or(ServerError::Fuck)?;
                    if !f.is_public {
                        return Err(ServerError::Fuck);
                    }
                    f.is_public = false;
                }
                SetPub(path) => {
                    let f = problem_files.get_mut(&path).ok_or(ServerError::Fuck)?;
                    if f.is_public {
                        return Err(ServerError::Fuck);
                    }
                    f.is_public = true;
                }
                Remove(path) => {
                    let _ = problem_files.remove(&path).ok_or(ServerError::Fuck)?;
                    let path = problem_file_path(&pid, &path);
                    tracing::trace!("remove {}", path.display());
                    fs::remove_file(&path)
                        .await
                        .map_err(ServerError::into_internal)?;
                }
                Upload { path, time, size } => {
                    if problem_files.contains_key(&path) {
                        let file = problem_files.get_mut(&path).unwrap();
                        file.last_modified = time;
                        file.size = size;
                    } else {
                        problem_files.insert(
                            path.clone(),
                            ProblemFile {
                                path: path.clone(),
                                uuid: uuid::Uuid::new_v4(),
                                is_public: false,
                                size,
                                last_modified: time,
                            },
                        );
                    }
                    let to = problem_file_path(&pid, &path);
                    let (_, tmp) = upload.next().ok_or(ServerError::Fuck)?;
                    tracing::trace!("move {} to {}", tmp.display(), to.display());
                    tokio::fs::copy(&tmp, &to)
                        .await
                        .map_err(ServerError::into_internal)?;
                    tokio::task::spawn_blocking(move || tmp.close())
                        .await
                        .unwrap()
                        .map_err(ServerError::into_internal)?;
                }
            }
        }

        let problem_files = Arc::new(problem_files.into_iter().map(|x| x.1).collect());
        let mut problem = get_problem(&pid).await?.as_ref().clone();
        problem.files = problem_files;
        set_problem(&pid, Arc::new(problem)).await?;

        drop(lock);
        Ok::<(), ServerError>(())
    })
    .await
    .unwrap()?;

    Ok(())
}
