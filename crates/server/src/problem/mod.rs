mod problem_lock;

use super::ServerError;
use dashmap::DashMap;
use problem_lock::ProblemLock;
use serde::{Deserialize, Serialize};
pub use shared::problem::*;
use shared::user::Uid;
use sqlx::{FromRow, Row, SqlitePool, sqlite::SqliteRow};
use static_init::dynamic;
use std::path::PathBuf;
use tokio::fs;
use tokio_util::io::ReaderStream;

#[dynamic]
static PROBLEM_LOCKS: DashMap<Pid, ProblemLock> = DashMap::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub pid: Pid,
    pub owner: Uid,
    pub title: String,
    pub statement: String,
    pub memory_limit: u32,
    pub time_limit: u32,
    pub testcases: Vec<Testcase>,
    pub files: Vec<ProblemFile>,
}

impl FromRow<'_, SqliteRow> for Problem {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        shared::from_json_in_row(row)
    }
}

impl Problem {
    pub async fn insert_db(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO problems (pid,owner,json) VALUES ($1,$2,$3)")
            .bind(self.pid.0.as_str())
            .bind(self.owner.0 as i64)
            .bind(serde_json::to_string(self).unwrap())
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub async fn read_problem(pid: &Pid) -> Result<Problem, ServerError> {
    let path = dirs::home_dir()
        .unwrap()
        .join("mygoj")
        .join("problems")
        .join(&pid.0)
        .join("config.json");
    if !path.exists() {
        return Err(ServerError::NotFound);
    }
    let json = fs::read_to_string(&path)
        .await
        .map_err(ServerError::into_internal)?;
    let problem: Problem = serde_json::from_str(&json).map_err(ServerError::into_internal)?;
    Ok(problem)
}

pub async fn get_problem_front(pid: &Pid) -> Result<ProblemFront, ServerError> {
    let problem = read_problem(pid).await?;

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
    let problem = read_problem(&pid).await?;
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
