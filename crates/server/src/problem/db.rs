use super::*;
use crate::db::DB;
use sqlx::{FromRow, SqlitePool, sqlite::SqliteRow};

impl FromRow<'_, SqliteRow> for Problem {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        shared::from_json_in_row(row)
    }
}

impl Problem {
    pub async fn insert_db(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let pid = self.pid.0.as_str();
        let owner = self.owner.map(|x| x.0 as i64);
        let json = serde_json::to_string(self).unwrap();
        sqlx::query!(
            "INSERT INTO problems (pid,owner,json) VALUES ($1,$2,$3)",
            pid,
            owner,
            json
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

pub async fn get_problem(pid: &Pid) -> Result<Arc<Problem>, sqlx::Error> {
    tracing::trace!("DB fetch problem {pid}");
    let pid = pid.0.as_str();
    let json = sqlx::query!("SELECT (json) FROM problems WHERE pid=$1", pid)
        .fetch_one(DB.get().unwrap())
        .await?
        .json
        .unwrap();

    let p = serde_json::from_str(&json).unwrap();
    Ok(Arc::new(p))
}

pub async fn set_problem(pid: &Pid, problem: &Problem) -> Result<(), sqlx::Error> {
    let pid = pid.0.as_str();
    let json = serde_json::to_string(&problem).unwrap();
    let owner = problem.owner.map(|x| x.0 as i64);
    let db = DB.get().unwrap();
    sqlx::query!(
        "UPDATE problems SET owner=$2,json=$3 WHERE pid=$1",
        pid,
        owner,
        json
    )
    .execute(db)
    .await?;
    Ok(())
}
