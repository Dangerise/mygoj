mod db;
mod front;
mod judge;
mod problem;
mod record;
mod user;
mod error;

pub mod init;
pub mod serve;

use shared::error::ServerError;
use std::path::PathBuf;
use error::Fuck;

#[track_caller]
pub fn storage_dir() -> PathBuf {
    dirs::home_dir().unwrap().join("mygoj")
}
