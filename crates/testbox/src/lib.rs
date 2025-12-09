use std::ffi::OsStr;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod error;
pub use error::Error;
pub(crate) use error::map_err;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(any(target_os = "linux")))]
compile_error!("The target platform is not supported, only on linux");

#[derive(Debug, Clone)]
pub struct RunResult {
    pub time_used: Duration,
    pub memory_used: u64,
    pub exit_code: Option<i32>,
    pub status: Status,
    pub stdout: Vec<u8>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(target_os = "linux")]
pub type PlatformTestBox = linux::LinuxTestBox;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Okay,
    RuntimeError,
    TimeLimitExceed,
    MemoryLimitExceed,
}

pub trait TestBox
where
    Self: Sized,
{
    fn new(config: &Config) -> impl Future<Output = Result<Self>>;
    fn run_single<'a>(
        &self,
        path: impl AsRef<Path>,
        args: impl IntoIterator<Item = &'a OsStr>,
        stdin: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<RunResult>>;
}

#[derive(Debug, Clone)]
pub struct Config {
    pub root: PathBuf,
    pub memory_limit: u64,
    pub time_limit: Duration,
}
