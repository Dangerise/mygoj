mod cache;
mod db;

use super::ServerError;
use super::judge::JUDGE_QUEUE;
use super::problem::{get_problem, problem_read_lock, problem_read_unlock};
use dashmap::DashMap;
use shared::judge::SingleJudgeResult;
use shared::record::*;
use shared::submission::Submission;
use shared::user::Uid;
use static_init::dynamic;
use tokio::sync::broadcast;

use axum::extract::{
    Query, WebSocketUpgrade,
    ws::{Message as WsMessage, WebSocket},
};
use axum::response::Response;

const LIMIT: u64 = 1 << 15;

struct RecordChannel {
    tx: broadcast::Sender<RecordMessage>,
    _rx: broadcast::Receiver<RecordMessage>,
}

impl RecordChannel {
    fn new(size: usize) -> Self {
        let (tx, rx) = broadcast::channel(size);
        Self { tx, _rx: rx }
    }
}

#[dynamic]
static JUDGING_RECORDS: DashMap<Rid, Record> = DashMap::new();

#[dynamic]
static RECORD_CHANNEL: DashMap<Rid, RecordChannel> = DashMap::new();

pub async fn new_record(rid: Rid, record: Record) -> Result<(), ServerError> {
    let pid = &record.pid;
    let case_count = get_problem(pid).await?.testcases.len();
    problem_read_lock(pid).await;
    JUDGING_RECORDS.insert(rid, record);
    RECORD_CHANNEL.insert(rid, RecordChannel::new(case_count * 2));
    {
        let mut queue = JUDGE_QUEUE.lock().await;
        queue.push_back(rid);
    }
    Ok(())
}

pub async fn submit(uid: Uid, submission: Submission) -> Result<Rid, ServerError> {
    let Submission { code, .. } = &submission;
    if code.len() > (50 << 10) {
        return Err(ServerError::Fuck);
    }

    let record = db::submit(uid, submission)
        .await
        .map_err(ServerError::into_internal)?;
    let rid = record.rid;
    cache::new_record(record.clone()).await;
    new_record(rid, record).await?;
    Ok(rid)
}

pub async fn get_record(rid: Rid) -> Result<Record, ServerError> {
    if let Some(rec) = cache::get_record(rid).await {
        return Ok(rec);
    }
    db::get_record(rid)
        .await
        .map_err(ServerError::into_internal)?
        .ok_or(ServerError::NotFound)
}

pub async fn update_record_single(
    rid: Rid,
    idx: usize,
    res: SingleJudgeResult,
) -> Result<(), ServerError> {
    let mut record = JUDGING_RECORDS.get_mut(&rid).unwrap();
    let RecordStatus::Running(status) = &mut record.status else {
        unreachable!()
    };

    let single = status.get_mut(idx).unwrap();
    assert!(single.is_none());
    *single = Some(res.clone());

    cache::update_record(rid, record.clone()).await;

    drop(record);

    let sender = &RECORD_CHANNEL.get(&rid).unwrap().tx;
    sender
        .send(RecordMessage::NewSingleResult(idx, res))
        .unwrap();

    Ok(())
}

pub async fn update_record(rid: Rid, status: RecordStatus) -> Result<(), ServerError> {
    tracing::info!("update rid {} {:#?}", rid, &status);

    use tokio::sync::broadcast::Sender;
    let send = |sender: &Sender<RecordMessage>| {
        let status = status.clone();
        let msg = match status {
            RecordStatus::Compiling => RecordMessage::Compiling,
            RecordStatus::CompileError(ce) => RecordMessage::CompileError(ce),
            RecordStatus::Running(v) => RecordMessage::Compiled(v.len()),
            RecordStatus::Completed(all) => RecordMessage::Completed(all),
            RecordStatus::Waiting => unreachable!(),
        };
        sender.send(msg).unwrap();
    };

    if status.done() {
        problem_read_unlock(&get_record(rid).await?.pid);
        let (_, mut record) = JUDGING_RECORDS.remove(&rid).unwrap();
        record.status = status.clone();

        cache::update_record(rid, record.clone()).await;
        db::update_record(rid, &record)
            .await
            .map_err(ServerError::into_internal)?;

        let channel = RECORD_CHANNEL.remove(&rid).unwrap().1;
        send(&channel.tx);
    } else {
        let mut record = JUDGING_RECORDS.get_mut(&rid).unwrap();
        record.status = status.clone();
        cache::update_record(rid, record.clone()).await;
        drop(record);

        let sender = RECORD_CHANNEL.get(&rid).unwrap();
        let sender = &sender.tx;
        send(sender);
    }

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct Qrid {
    rid: u64,
}

pub async fn ws(
    ws: WebSocketUpgrade,
    Query(Qrid { rid }): Query<Qrid>,
) -> Result<Response, ServerError> {
    let rid = Rid(rid);
    tracing::info!("establish connect for {rid}");
    let record = get_record(rid).await?;
    if record.status.done() {
        return Err(ServerError::Fuck);
    }
    let resp = ws.on_upgrade(move |socket| handle_socket(socket, rid));
    Ok(resp)
}

async fn handle_socket(mut socket: WebSocket, rid: Rid) {
    let Some(mut receiver) = RECORD_CHANNEL.get(&rid).map(|x| x.tx.subscribe()) else {
        return;
    };
    tracing::info!("handle connect for {rid}");
    loop {
        let Ok(msg) = receiver.recv().await else {
            return;
        };
        let res = socket
            .send(WsMessage::Text(
                serde_json::to_string_pretty(&msg).unwrap().into(),
            ))
            .await;
        if res.is_err() {
            return;
        }
    }
}
