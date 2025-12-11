use salvo::http::HeaderMap;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

pub use shared::problem::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub title: String,
    pub statement: String,
    pub memory_limit: u32,
    pub time_limit: u32,
    pub testcases: Vec<Testcase>,
    pub files: Vec<ProblemFile>,
}

pub async fn read_problem(pid: &Pid) -> eyre::Result<Problem> {
    let path = dirs::home_dir()
        .unwrap()
        .join("mygoj")
        .join("problems")
        .join(&pid.0)
        .join("config.json");
    let json = fs::read_to_string(&path).await?;
    let problem: Problem = serde_json::from_str(&json)?;
    Ok(problem)
}

#[handler]
pub async fn problem_front(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let pid = Pid(req.query("pid").unwrap());

    let problem = read_problem(&pid).await?;

    let front = ProblemFront {
        title: problem.title,
        statement: problem.statement,
        time_limit: problem.time_limit,
        memory_limit: problem.memory_limit,
        pid,
    };

    resp.render(Json(front));
    Ok(())
}

pub async fn problem_data(pid: Pid) -> eyre::Result<ProblemData> {
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

pub async fn send_problem_file(pid: Pid, filename: &str, resp: &mut Response) -> eyre::Result<()> {
    let empty_header_map = HeaderMap::new();
    resp.send_file(
        problem_file_path(&pid, filename),
        &empty_header_map,
    )
    .await;
    Ok(())
}
