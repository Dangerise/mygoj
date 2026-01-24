mod cache;
mod db;
pub mod files;

use super::{Fuck, ServerError};
use compact_str::CompactString;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
pub use shared::problem::*;
use shared::user::{LoginedUser, Uid};
use static_init::dynamic;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

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

fn problem_storage_path(pid: &Pid) -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join("mygoj")
        .join("problems")
        .join(&pid.0)
}

async fn get_problem_file(pid: &Pid, path: &str) -> Result<PathBuf, ServerError> {
    let storage = problem_storage_path(pid);
    let uuid = get_problem(pid)
        .await?
        .files
        .iter()
        .find_map(|d| (d.path == path).then_some(d.uuid))
        .ok_or(ServerError::NotFound)?;
    Ok(storage.join(uuid.to_string()))
}

use axum::body::Body;
use axum::response::Response;

pub async fn send_problem_file(pid: Pid, filename: &str) -> Result<Response, ServerError> {
    let path = get_problem_file(&pid, filename).await?;
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
