use super::*;
use papaya::HashMap;

#[dynamic]
pub static RECORDS: HashMap<Rid, Record> = HashMap::new();

pub async fn new_record(record: Record) {
    RECORDS.pin().insert(record.rid, record);
}

pub async fn get_record(rid: Rid) -> Option<Record> {
    RECORDS.pin().get(&rid).cloned()
}

pub async fn update_record(rid: Rid, record: Record) {
    RECORDS.pin().insert(rid, record);
}
