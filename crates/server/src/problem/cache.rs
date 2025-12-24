use super::*;
use moka::future::Cache;

#[dynamic]
static PROBLEMS: Cache<Pid, Problem> = Cache::new(1 << 10);

pub async fn get_problem(pid: &Pid) -> Option<Problem> {
    PROBLEMS.get(pid).await
}

pub async fn update_problem(pid: &Pid, prob: Problem) {
    PROBLEMS.insert(pid.clone(), prob).await;
}
