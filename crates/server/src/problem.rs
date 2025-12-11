use salvo::prelude::*;
use serde::{Deserialize, Serialize};
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

#[handler]
pub async fn problem_front(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let pid = Pid(req.query("pid").unwrap());

    let path = dirs::home_dir()
        .unwrap()
        .join("mygoj")
        .join("problems")
        .join(&pid.0)
        .join("config.json");

    tracing::info!("read problem config file {}", path.display());

    let config = fs::read_to_string(&path).await?;
    let problem: Problem = serde_json::from_str(&config)?;

    let front = ProblemFront {
        title: problem.title.clone(),
        statement: problem.statement.clone(),
        time_limit: problem.time_limit,
        memory_limit: problem.memory_limit,
        pid,
    };

    tracing::info!("response problem front {:?}", &front);

    resp.render(Json(front));
    Ok(())
}
