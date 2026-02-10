use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize, Clone)]
pub enum ServerError {
    #[error("user not found")]
    UserNotFound,
    #[error("wrong password")]
    PasswordWrong,
    #[error("Fuck you")]
    Fuck,
    #[error("login out dated")]
    LoginOutDated,
    #[error("internal error {0:#?}")]
    Internal(String),
    #[error("not found ")]
    NotFound,
    #[error("the email has exist")]
    EmailExist,
    #[error("username exist")]
    UsernameExist,
    #[error("no privilege")]
    NoPrivilege,
    #[error("network error")]
    Network,
    #[error("bad data")]
    BadData,
}

#[cfg(feature = "server")]
mod on_server {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    impl IntoResponse for ServerError {
        fn into_response(self) -> Response {
            let json = serde_json::to_string_pretty(&self).unwrap();
            Response::builder()
                .status(self.status_code())
                .body(Body::new(json))
                .unwrap()
        }
    }

    impl ServerError {
        pub fn into_internal<E: Into<eyre::Report>>(err: E) -> Self {
            let report: eyre::Report = err.into();
            Self::Internal(format!("{:#?}", report))
        }
        pub fn status_code(&self) -> StatusCode {
            use ServerError::*;
            match self {
                Network => StatusCode::BAD_REQUEST,
                UserNotFound | PasswordWrong | Fuck | EmailExist | UsernameExist => {
                    StatusCode::BAD_REQUEST
                }
                NoPrivilege => StatusCode::FORBIDDEN,
                LoginOutDated => StatusCode::UNAUTHORIZED,
                Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
                NotFound => StatusCode::NOT_FOUND,
                BadData => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }
}
