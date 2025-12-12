use salvo::prelude::*;
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
static SIGNALS: Mutex<HashMap<Uuid, JudgeSignal>> = Mutex::new(HashMap::new());

pub async fn check_alive() {
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

use super::problem::{problem_data, send_problem_file};
use super::record::{get_record, update_record};

#[handler]
pub async fn receive_message(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let msg: JudgeMessage = req.parse_json().await?;
    match msg {
        JudgeMessage::Signal(sig) => {
            let command = receive_signal(sig).await?;
            resp.render(Json(command));
        }

        JudgeMessage::GetProblemData(pid) => {
            let problem_data = problem_data(pid).await?;
            resp.render(Json(problem_data));
        }
        JudgeMessage::GetProblemFile(pid, filename) => {
            send_problem_file(pid, &filename, resp).await?
        }
        JudgeMessage::GetRecord(rid) => {
            let record = get_record(rid).await?;
            resp.render(Json(record));
        }
        JudgeMessage::SendCompileResult(rid, res) => {
            let status = match res {
                CompileResult::Compiled => RecordStatus::Running,
                CompileResult::Error(ce) => RecordStatus::CompileError(ce),
            };
            update_record(rid, status).await?;
            resp.render(Json(()));
        }
        JudgeMessage::SendAllJudgeResults(rid, res) => {
            update_record(rid, RecordStatus::Completed(res)).await?;
            resp.render(Json(()));
        }
    }
    Ok(())
}

pub async fn receive_signal(signal: JudgeSignal) -> eyre::Result<JudgeCommand> {
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

#[handler]
pub async fn judge_machines(_req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    resp.render(Json(
        SIGNALS
            .lock()
            .await
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<_>>(),
    ));
    Ok(())
}
