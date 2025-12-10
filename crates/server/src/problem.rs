use eyre::eyre;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::fs;

pub use shared::problem::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Testcase {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub front: ProblemFront,
    pub testcases: Vec<Testcase>,
}

#[handler]
pub async fn problem_front(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let pid = req
        .query::<String>("pid")
        .ok_or_else(|| eyre!("pid nod found"))?;

    let path = dirs::home_dir()
        .unwrap()
        .join("mygoj")
        .join("problems")
        .join(&pid)
        .join("config.json");

    tracing::info!("read problem config file {}", path.display());

    let config = fs::read_to_string(&path).await?;
    let mut problem: Problem = serde_json::from_str(&config)?;
    problem.front.pid = Pid(pid);

    tracing::info!("response problem front {:?}", &problem.front);

    resp.render(Json(problem.front));
    Ok(())
}
