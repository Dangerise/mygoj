use super::*;
use papaya::HashMap;
use std::sync::Arc;

#[dynamic]
static PROBLEMS: HashMap<Pid, Arc<Problem>> = HashMap::new();

pub async fn get_problem(pid: &Pid) -> Option<Arc<Problem>> {
    PROBLEMS.pin().get(pid).cloned()
}

pub async fn update_problem(pid: &Pid, prob: Arc<Problem>) {
    PROBLEMS.pin().insert(pid.clone(), prob);
}
