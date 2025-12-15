use super::ServerError;
use super::problem::problem_read_unlock;
use shared::judge::SingleJudgeResult;
use shared::record::*;
use static_init::dynamic;
use tokio::sync::RwLock;

#[dynamic]
pub static RECORDS: RwLock<Vec<Record>> = RwLock::new(Vec::new());

pub async fn get_record(rid: Rid) -> Result<Record, ServerError> {
    tracing::info!("query rid {rid}");
    let records = RECORDS.read().await;
    let record = records.get(rid.0 as usize).unwrap();
    tracing::info!("record {:?}", record);
    Ok(record.clone())
}

pub async fn update_record_single(
    rid: Rid,
    idx: usize,
    res: SingleJudgeResult,
) -> Result<(), ServerError> {
    let mut records = RECORDS.write().await;
    let record = records.get_mut(rid.0 as usize).unwrap();
    if let RecordStatus::Running(status) = &mut record.status {
        let single = status.get_mut(idx).unwrap();
        assert!(single.is_none());
        *single = Some(res);
    }
    Ok(())
}

pub async fn update_record(rid: Rid, status: RecordStatus) -> Result<(), ServerError> {
    tracing::info!("update rid {} {:#?}", rid, &status);
    if matches!(
        status,
        RecordStatus::Completed(_) | RecordStatus::CompileError(_)
    ) {
        problem_read_unlock(&get_record(rid).await?.pid);
    }
    let mut records = RECORDS.write().await;
    let record = records.get_mut(rid.0 as usize).unwrap();
    record.status = status;
    Ok(())
}
