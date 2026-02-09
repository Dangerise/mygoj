use super::judge::judge_machines;
use super::problem::{
    can_manage_problem, files::require_problem_file_download_token, get_problem,
    get_problem_editable, get_problem_front,
};
use super::record::{get_record, submit};
use super::user::{get_user_login, remove_token, user_login, user_register};
use super::{Fuck, ServerError};
use rust_embed::RustEmbed;
use shared::front::FrontMessage;
use shared::problem::Pid;
use shared::token::Token;
use std::borrow::Cow;
use std::sync::LazyLock;

use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, Response};

#[cfg(debug_assertions)]
#[derive(RustEmbed)]
#[folder = "../../target/dx/front/debug/web/public"]
struct FrontResource;

#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "../../target/dx/front/release/web/public"]
struct FrontResource;

static INDEX: LazyLock<String> = LazyLock::new(|| {
    let data = FrontResource::get("index.html").unwrap().data;
    let html = String::from_utf8_lossy(&data);
    html.into_owned()
});

pub async fn index() -> Html<&'static str> {
    let file: &str = INDEX.as_str();
    Html(file)
}

use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum_extra::body::AsyncReadBody;
async fn dir(path: String) -> Result<Response, StatusCode> {
    // Ok(path.into_response())
    let data = match FrontResource::get(&path).ok_or(StatusCode::NOT_FOUND)?.data {
        Cow::Borrowed(r) => r,
        Cow::Owned(_) => unreachable!(),
    };
    let body = AsyncReadBody::new(data);
    let ty = std::path::Path::new(&path)
        .extension()
        .map(|ext| {
            let ext = String::from_utf8_lossy(ext.as_encoded_bytes());
            let ext = &*ext;
            match ext {
                "js" => "text/javascript",
                "wasm" => "application/wasm",
                _ => "",
            }
        })
        .unwrap_or("");
    // tracing::info!("{} for {}", ty, path);
    Ok(([(CONTENT_TYPE, ty)], body).into_response())
}

pub async fn assets(Path(path): Path<String>) -> Result<Response, StatusCode> {
    dir(format!("assets/{path}")).await
}

pub async fn wasm(Path(path): Path<String>) -> Result<Response, StatusCode> {
    dir(format!("wasm/{path}")).await
}

use axum::Extension;
use axum::extract::Request;
use axum::middleware::Next;
use axum_extra::typed_header::TypedHeader;
use headers::authorization::{Authorization, Basic, Bearer};
pub async fn logined_user_layer(
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ServerError> {
    let login = if let Some(auth) = auth {
        let token = auth.token();
        let token = Token::decode(token).fuck()?;
        let login = get_user_login(token).await?;
        Some(login)
    } else {
        None
    };
    tracing::trace!("auth {login:?}");
    request.extensions_mut().insert(login);
    let resp = next.run(request).await;
    Ok(resp)
}

pub async fn login(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
) -> Result<Response, ServerError> {
    let ident = auth.username().into();
    let pwd = auth.password().into();
    let (token, logined_user) = user_login(ident, pwd).await?;
    let resp = Json((token.encode(), logined_user)).into_response();
    Ok(resp)
}

pub async fn logout(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ServerError> {
    let token = Token::decode(auth.token()).fuck()?;
    remove_token(token).await?;
    Ok(())
}

use shared::user::LoginedUser;
pub async fn receive_front_message(
    Extension(logined_user): Extension<Option<LoginedUser>>,
    Json(message): Json<FrontMessage>,
) -> Result<Response, ServerError> {
    fn to_json<T: serde::Serialize>(val: T) -> Result<Response, ServerError> {
        Ok(Response::new(Body::new(
            serde_json::to_string_pretty(&val).map_err(ServerError::into_internal)?,
        )))
    }
    let can_edit_problem = async |pid: &Pid| {
        if let Some(user) = &logined_user
            && can_manage_problem(user, pid).await? {
                return Ok(());
            }
        Err(ServerError::Fuck)
    };

    match message {
        FrontMessage::GetProblemFiles(pid) => {
            can_edit_problem(&pid).await?;
            let files = get_problem(&pid).await.map(|x| x.files.clone())?;
            to_json(&files)
        }
        FrontMessage::GetProblemEditable(pid) => {
            can_edit_problem(&pid).await?;
            let editable = get_problem_editable(&pid).await?;
            to_json(&editable)
        }
        FrontMessage::GetProblemFront(pid) => {
            let front = get_problem_front(&pid).await?;
            to_json(&front)
        }
        FrontMessage::CheckJudgeMachines => {
            let res = judge_machines().await?;
            to_json(&res)
        }
        FrontMessage::GetRecord(rid) => {
            let rec = get_record(rid).await?;
            to_json(&rec)
        }
        FrontMessage::Submit(submission) => {
            let uid = logined_user.map(|x| x.uid).fuck()?;
            let rid = tokio::spawn(submit(uid, submission)).await.unwrap()?;
            to_json(rid)
        }
        FrontMessage::RegisterUser(registration) => {
            let uid = tokio::spawn(user_register(registration)).await.unwrap()?;
            to_json(uid)
        }
        FrontMessage::GetLoginedUser => to_json(&logined_user),
        FrontMessage::RequireProblemFileDownloadToken(pid, path) => {
            let token = tokio::spawn(require_problem_file_download_token(logined_user, pid, path))
                .await
                .unwrap()?;
            to_json(token)
        }
    }
}
