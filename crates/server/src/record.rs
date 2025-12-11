use salvo::prelude::*;
use shared::record::*;
use static_init::dynamic;
use tokio::sync::RwLock;

#[dynamic]
pub static RECORDS: RwLock<Vec<Record>> = RwLock::new(Vec::new());

pub async fn get_record(rid: Rid) -> eyre::Result<Record> {
    tracing::info!("query rid {rid}");
    let records = RECORDS.read().await;
    let record = records.get(rid.0 as usize).unwrap();
    tracing::info!("record {:?}", record);
    Ok(record.clone())
}

pub async fn update_record(rid: Rid, status: RecordStatus) -> eyre::Result<()> {
    tracing::info!("update rid {} {:#?}", rid, &status);
    let mut records = RECORDS.write().await;
    let record = records.get_mut(rid.0 as usize).unwrap();
    record.status = status;
    Ok(())
}

#[handler]
pub async fn get_record_handler(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let rid: Rid = Rid(req.query("rid").unwrap());
    let record = get_record(rid).await?;
    resp.render(Json(record));
    Ok(())
}
