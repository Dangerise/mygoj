use serde::{Deserialize, Serialize};

pub mod constant;
pub mod error;
pub mod front;
pub mod judge;
pub mod problem;
pub mod record;
pub mod submission;
pub mod token;
pub mod user;
pub mod download;

// use token::*;
use judge::*;
use problem::*;
use record::*;
use submission::*;
use user::*;

#[cfg(feature = "server")]
pub fn from_json_in_row<T: serde::de::DeserializeOwned>(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<T, sqlx::Error> {
    use sqlx::Row;
    let s = row.get("json");
    serde_json::from_str(s).map_err(|err| sqlx::Error::Decode(Box::new(err)))
}
