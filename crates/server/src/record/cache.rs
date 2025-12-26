use super::*;

use moka::future::Cache;

#[dynamic]
pub static RECORDS: Cache<Rid, Record> = Cache::new(LIMIT);

pub async fn new_record(record: Record) {
    RECORDS.insert(record.rid, record).await
}

pub async fn get_record(rid: Rid) -> Option<Record> {
    RECORDS.get(&rid).await
}

pub async fn update_record(rid: Rid, record: Record) {
    RECORDS.insert(rid, record).await;
}
