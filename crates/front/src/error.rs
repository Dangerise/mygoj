use super::*;

pub enum ErrorKind {
    Server(ServerError),
    Client(eyre::Report),
}

impl From<eyre::Report> for ErrorKind {
    fn from(value: eyre::Report) -> Self {
        value
            .downcast()
            .map(ErrorKind::Server)
            .unwrap_or_else(ErrorKind::Client)
    }
}

pub trait Split {
    type Output;
    fn split(self) -> Self::Output;
}
impl Split for eyre::Report {
    type Output = ErrorKind;
    fn split(self) -> Self::Output {
        self.into()
    }
}
impl<T> Split for eyre::Result<T> {
    type Output = Result<T, ErrorKind>;
    fn split(self) -> Self::Output {
        self.map_err(Into::into)
    }
}
