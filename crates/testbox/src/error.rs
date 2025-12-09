#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sys error {0:?}")]
    Io(#[from] std::io::Error),
    #[error("sys error {0:?}")]
    Errno(#[from] nix::Error),
    #[error("Other error {0:?}")]
    OtherError(String),
}

pub fn map_err(err: impl std::fmt::Debug) -> Error {
    Error::OtherError(format!("{err:?}"))
}
