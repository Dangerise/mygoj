use super::*;
use crate::DB;
use sqlx::{FromRow, SqlitePool, sqlite::SqliteRow};

impl FromRow<'_, SqliteRow> for Problem {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        shared::from_json_in_row(row)
    }
}

impl Problem {
    pub async fn insert_db(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO problems (pid,owner,json) VALUES ($1,$2,$3)")
            .bind(self.pid.0.as_str())
            .bind(self.owner.map(|x| x.0 as i64))
            .bind(serde_json::to_string(self).unwrap())
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub async fn get_problem(pid: &Pid) -> Result<Problem, sqlx::Error> {
    tracing::trace!("DB fetch problem {pid}");
    let ret = sqlx::query_as("SELECT (json) FROM problems WHERE pid=$1")
        .bind(pid.0.as_str())
        .fetch_one(DB.get().unwrap())
        .await?;
    Ok(ret)
}
