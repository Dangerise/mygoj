use super::*;
use papaya::HashMap;

#[dynamic]
static PROBLEMS: HashMap<Pid, Problem> = HashMap::new();

pub async fn get_problem(pid: &Pid) -> Option<Problem> {
    PROBLEMS.pin().get(pid).cloned()
}

pub async fn update_problem(pid: &Pid, prob: Problem) {
    PROBLEMS.pin().insert(pid.clone(), prob);
}
