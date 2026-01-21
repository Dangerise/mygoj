use super::ServerError;

pub trait Fuck {
    type Output;
    fn fuck(self) -> Result<Self::Output, ServerError>;
}

impl<T> Fuck for Option<T> {
    type Output = T;
    fn fuck(self) -> Result<T, ServerError> {
        self.ok_or(ServerError::Fuck)
    }
}

impl<T, E> Fuck for Result<T, E> {
    type Output = T;
    fn fuck(self) -> Result<Self::Output, ServerError> {
        self.map_err(|_| ServerError::Fuck)
    }
}
