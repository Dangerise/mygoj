use shared::judge::*;
use shared::record::*;
use static_init::dynamic;
use std::collections::{HashMap, VecDeque, hash_map};
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

#[dynamic]
pub static JUDGE_QUEUE: Mutex<VecDeque<Rid>> = Mutex::new(VecDeque::new());

#[dynamic]
static SIGNALS: Mutex<HashMap<Uuid, JudgeMachineSignal>> = Mutex::new(HashMap::new());

pub async fn track_judge_machines() {
    let mut to_remove = Vec::new();
    loop {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let mut signals = SIGNALS.lock().await;
        to_remove.clear();
        for sig in signals.values() {
            if now - sig.timestamp > 4000 {
                to_remove.push(sig.uuid);
            }
        }
        for uuid in &to_remove {
            signals.remove(uuid);
            tracing::info!("judge machine offline {}", uuid);
        }
        drop(signals);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn generate_command() -> eyre::Result<JudgeCommand> {
    let mut queue = JUDGE_QUEUE.lock().await;
    if queue.is_empty() {
        Ok(JudgeCommand::Null)
    } else {
        let rid = queue.pop_front().unwrap();
        Ok(JudgeCommand::Judge(rid))
    }
}

use super::EyreResult;
use super::problem::{problem_data, send_problem_file};
use super::record::{get_record, update_record, update_record_single};
use axum::Json;
use axum::body::Body;
use axum::response::Response;

pub async fn receive_message(Json(msg): Json<JudgeMessage>) -> EyreResult<Response> {
    fn to_json<T: serde::Serialize>(val: T) -> EyreResult<Response> {
        Ok(Response::new(Body::new(serde_json::to_string_pretty(
            &val,
        )?)))
    }
    match msg {
        JudgeMessage::Signal(sig) => {
            let command = receive_signal(sig).await?;
            to_json(command)
        }
        JudgeMessage::GetProblemData(pid) => {
            let problem_data = problem_data(pid).await?;
            to_json(problem_data)
        }
        JudgeMessage::GetProblemFile(pid, filename) => {
            send_problem_file(pid, &filename).await
        }
        JudgeMessage::GetRecord(rid) => {
            let record = get_record(rid).await?;
            to_json(record)
        }
        JudgeMessage::SendSingleJudgeResult(rid, idx, res) => {
            update_record_single(rid, idx, res).await?;
            to_json(())
        }
        JudgeMessage::SendCompileResult(rid, res) => {
            let status = match res {
                CompileResult::Compiled => {
                    RecordStatus::Running(vec![
                        const { None };
                        problem_data(get_record(rid).await?.pid)
                            .await?
                            .testcases
                            .len()
                    ])
                }
                CompileResult::Error(ce) => RecordStatus::CompileError(ce),
            };
            update_record(rid, status).await?;
            to_json(())
        }
        JudgeMessage::SendAllJudgeResults(rid, res) => {
            update_record(rid, RecordStatus::Completed(res)).await?;
            to_json(())
        }
    }
}

pub async fn receive_signal(signal: JudgeMachineSignal) -> eyre::Result<JudgeCommand> {
    let uuid = signal.uuid;
    let mut signals = SIGNALS.lock().await;
    tracing::info!("received signal {:?}", &signal);
    if let hash_map::Entry::Vacant(e) = signals.entry(uuid) {
        tracing::info!("new judge machine online {}", uuid);
        e.insert(signal);
    } else {
        *signals.get_mut(&uuid).unwrap() = signal;
    }

    let command = generate_command().await?;
    Ok(command)
}

pub async fn judge_machines() -> eyre::Result<Vec<JudgeMachineSignal>> {
    let res = SIGNALS
        .lock()
        .await
        .iter()
        .map(|x| x.1.clone())
        .collect::<Vec<_>>();
    Ok(res)
}
