mod cache;
mod db;
mod problem_lock;

use super::ServerError;
use dashmap::DashMap;
use problem_lock::ProblemLock;
use serde::{Deserialize, Serialize};
pub use shared::problem::*;
use shared::user::Uid;
use static_init::dynamic;
use std::path::PathBuf;
use tokio::fs;
use tokio_util::io::ReaderStream;

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

pub async fn get_problem_front(pid: &Pid) -> Result<ProblemFront, ServerError> {
    let problem = get_problem(pid).await?;

    let front = ProblemFront {
        title: problem.title,
        statement: problem.statement,
        time_limit: problem.time_limit,
        memory_limit: problem.memory_limit,
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

#[allow(dead_code)]
pub async fn probllm_write_lock(pid: &Pid) {
    PROBLEM_LOCKS
        .entry(pid.clone())
        .or_default()
        .write_lock()
        .await
}

#[allow(dead_code)]
pub fn probllm_write_unlock(pid: &Pid) {
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
