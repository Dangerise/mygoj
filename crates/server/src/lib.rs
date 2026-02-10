mod db;
mod error;
mod front;
mod judge;
mod problem;
mod record;
mod user;

pub mod init;
pub mod serve;

use error::Fuck;
use shared::error::ServerError;
use std::path::PathBuf;

#[track_caller]
pub fn storage_dir() -> PathBuf {
    dirs::home_dir().unwrap().join("mygoj")
}
