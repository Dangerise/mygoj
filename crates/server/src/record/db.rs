use super::*;
use crate::db::DB;

pub async fn get_record(rid: Rid) -> Result<Option<Record>, sqlx::Error> {
    let db = DB.get().unwrap();
    let rid = rid.0 as i64;
    let ret = sqlx::query!("SELECT (json) FROM records WHERE rid=$1", rid)
        .fetch_optional(db)
        .await?
        .map(|rec| serde_json::from_str(&rec.json.unwrap()).unwrap());
    Ok(ret)
}

pub async fn update_record(rid: Rid, record: &Record) -> Result<(), sqlx::Error> {
    let db = DB.get().unwrap();
    assert!(record.status.done());
    let json = serde_json::to_string(record).unwrap();
    let rid = rid.0 as i64;
    let flag = record.status.flag().as_str();
    sqlx::query!(
        "UPDATE records SET json=$1,flag=$2 WHERE rid=$3",
        json,
        flag,
        rid
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn submit(uid: Uid, Submission { code, pid }: Submission) -> Result<Record, sqlx::Error> {
    let db = DB.get().unwrap();
    let time = chrono::Utc::now().timestamp();
    let res = {
        let (pid, uid, flag) = (pid.0.as_str(), uid.0 as i64, RecordFlag::Waiting.as_str());
        sqlx::query!(
            "INSERT INTO records (pid,uid,flag,time) VALUES ($1,$2,$3,$4)",
            pid,
            uid,
            flag,
            time
        )
        .execute(db)
        .await?
    };
    let rid = res.last_insert_rowid() as u64;
    let rid = Rid(rid);
    let record = Record {
        rid,
        pid,
        uid,
        code,
        time,
        status: RecordStatus::Waiting,
    };
    let json = serde_json::to_string(&record).unwrap();
    let rid = rid.0 as i64;
    sqlx::query!("UPDATE records SET json=$1 WHERE rid=$2", json, rid)
        .execute(db)
        .await?;
    Ok(record)
}
