use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct EyreError(eyre::Report);

pub type EyreResult<T> = Result<T, EyreError>;

impl IntoResponse for EyreError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::new(format!("{:#?}", self.0)))
            .unwrap()
    }
}

impl<E> From<E> for EyreError
where
    E: Into<eyre::Report>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}
