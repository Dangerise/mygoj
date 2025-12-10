use salvo::prelude::*;
use shared::record::Record;
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub static RECORDS: LazyLock<RwLock<Vec<Record>>> = LazyLock::new(|| RwLock::new(Vec::new()));

#[handler]
pub async fn get_record(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let rid: u64 = req.query("rid").unwrap();

    tracing::info!("query rid {rid}");

    let records = RECORDS.read().await;
    let record = records.get(rid as usize).unwrap();
    resp.render(Json(record));

    tracing::info!("record {:?}", record);
    Ok(())
}
