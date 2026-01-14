mod cache;
mod db;
mod problem_lock;

use super::ServerError;
use problem_lock::ProblemLock;
use serde::{Deserialize, Serialize};
pub use shared::problem::*;
use shared::user::Uid;
use static_init::dynamic;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[dynamic]
static PROBLEM_LOCKS: DashMap<Pid, ProblemLock> = DashMap::new();

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Problem {
    pub pid: Pid,
    pub owner: Option<Uid>,
    pub title: String,
    pub statement: String,
    pub memory_limit: u32,
    pub time_limit: u32,
    pub testcases: Vec<Testcase>,
    pub files: Vec<ProblemFile>,
}

pub async fn get_problem(pid: &Pid) -> Result<Problem, ServerError> {
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

pub async fn set_problem(pid: &Pid, data: Problem) -> Result<(), ServerError> {
    db::set_problem(pid, data.clone())
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
        title: problem.title,
        statement: problem.statement,
        time_limit: problem.time_limit,
        memory_limit: problem.memory_limit,
        owner: problem.owner,
        public_files: problem
            .files
            .into_iter()
            .filter(|f| f.is_public)
            .map(|f| f.path)
            .collect(),
        pid: pid.clone(),
    };

    Ok(front)
}

pub async fn problem_read_lock(pid: &Pid) {
    PROBLEM_LOCKS
        .entry(pid.clone())
        .or_default()
        .read_lock()
        .await
}

pub async fn problem_write_lock(pid: &Pid) {
    PROBLEM_LOCKS
        .entry(pid.clone())
        .or_default()
        .write_lock()
        .await
}

pub fn problem_write_unlock(pid: &Pid) {
    PROBLEM_LOCKS.get(pid).unwrap().write_unlock()
}

pub fn problem_read_unlock(pid: &Pid) {
    PROBLEM_LOCKS.get(pid).unwrap().read_unlock()
}

pub async fn problem_data(pid: Pid) -> Result<ProblemData, ServerError> {
    let problem = get_problem(&pid).await?;
    let data = ProblemData {
        pid,
        files: problem.files,
        testcases: problem.testcases,
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

use axum::extract::{Path, Request};
use compact_str::CompactString;
use dashmap::{DashMap, DashSet};
use futures_util::StreamExt;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

#[derive(Debug)]
struct CommitChannel {
    to_upload: DashSet<CompactString>,
    pid: Pid,
    sender: mpsc::Sender<()>,
}

#[dynamic]
static COMMIT_CHANNELS: DashMap<Uuid, CommitChannel> = DashMap::new();

pub async fn upload_problem_file(
    Path(uuid): Path<&str>,
    Path(path): Path<CompactString>,
    request: Request,
) -> Result<(), ServerError> {
    let uuid: Uuid = uuid.parse().map_err(|_| ServerError::Fuck)?;

    let channel = COMMIT_CHANNELS.get_mut(&uuid).ok_or(ServerError::Fuck)?;
    channel.to_upload.remove(&path).ok_or(ServerError::Fuck)?;

    let tmpfile = tokio::task::spawn_blocking(|| tempfile::NamedTempFile::new())
        .await
        .unwrap()
        .map_err(ServerError::into_internal)?;
    let (file, tmppath): (std::fs::File, _) = tmpfile.into_parts();
    let file = tokio::fs::File::from_std(file);
    let writer = tokio::io::BufWriter::with_capacity(1 << 20, file);

    let mut stream = request.body().into_data_stream();
    while let Some(ret) = stream.next().await {
        let bytes = ret.map_err(ServerError::into_internal)?;
    }

    tokio::task::spawn_blocking(move || {
        drop(writer);
        drop(tmppath);
    })
    .await
    .unwrap();
    Ok(())
}

pub async fn commit_files_change(
    pid: Pid,
    evts: Vec<FileChangeEvent>,
) -> Result<Uuid, ServerError> {
    let mut files = get_problem(&pid)
        .await?
        .files
        .into_iter()
        .map(|f| (f.path.clone(), f))
        .collect::<BTreeMap<_, _>>();
    let to_upload = HashSet::new();
    for evt in evts {
        use FileChangeEvent::*;
        match evt {
            SetPriv(path) => {
                let file = files.get_mut(&path).ok_or(ServerError::Fuck)?;
                if !file.is_public {
                    return Err(ServerError::Fuck);
                }
                file.is_public = true;
            }
            SetPub(path) => {
                let file = files.get_mut(&path).ok_or(ServerError::Fuck)?;
                if file.is_public {
                    return Err(ServerError::Fuck);
                }
                file.is_public = false;
            }
            Upload(path) => {
                files.get_mut(&path).ok_or(ServerError::Fuck)?;
                to_upload.pin().insert(path);
            }
            Remove(path) => {
                files.get_mut(&path).ok_or(ServerError::Fuck)?;
                fs::remove_file(problem_file_path(&pid, &path))
                    .await
                    .map_err(ServerError::into_internal)?;
            }
        }
    }
    let files = files.into_iter().map(|(_, x)| x).collect::<Vec<_>>();
    let (sender, mut receiver) = mpsc::channel(to_upload.len());
    let to_upload_count = to_upload.len();
    let channel = CommitChannel {
        to_upload,
        pid: pid.clone(),
        sender,
    };
    let uuid = Uuid::new_v4();
    COMMIT_CHANNELS.pin().insert(uuid, channel);
    tokio::spawn(async move {
        let watch = async {
            let mut count = to_upload_count;
            if count == 0 {
                return;
            }
            loop {
                receiver.recv().await.unwrap();
                assert!(count >= 1);
                count -= 1;
                if count == 0 {
                    let ret = COMMIT_CHANNELS.pin().remove(&uuid).is_some();
                    assert!(ret);
                    return;
                }
            }
        };
        let ret = tokio::time::timeout(Duration::from_hours(1), watch).await;
        let pid;
        {
            let pin = COMMIT_CHANNELS.pin();
            let channel = pin.remove(&uuid).unwrap();
            let Ok(_) = ret else {
                tracing::error!("channel droped");
                return;
            };
            assert!(channel.to_upload.is_empty());
            pid = channel.pid.clone();
        }
        problem_write_lock(&pid).await;
        let Ok(mut prob) = get_problem(&pid).await else {
            tracing::error!("database access error get problem {pid}");
            return;
        };
        prob.files = files;
        let Ok(()) = set_problem(&pid, prob).await else {
            tracing::error!("database access error set problem {pid}");
            return;
        };
        problem_write_unlock(&pid);
    });
    Ok(uuid)
}
