use super::ServerError;
use super::judge::judge_machines;
use super::problem::{get_problem, get_problem_editable, get_problem_front};
use super::record::{get_record, submit};
use super::user::{get_user_login, remove_token, user_login, user_register};
use compact_str::CompactString;
use rust_embed::RustEmbed;
use shared::front::FrontMessage;
use shared::token::Token;
use std::borrow::Cow;
use std::sync::LazyLock;

use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
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
    // tracing::info!("assets {path}");
    dir(format!("assets/{path}")).await
}

pub async fn wasm(Path(path): Path<String>) -> Result<Response, StatusCode> {
    // tracing::info!("wasm {path}");
    dir(format!("wasm/{path}")).await
}

#[axum::debug_handler]
pub async fn receive_front_message(
    headers: HeaderMap,
    Json(message): Json<FrontMessage>,
) -> Result<Response, ServerError> {
    let id = headers.get("x-request-id").unwrap().to_str().unwrap();
    tracing::trace!("new front message {id}");
    tracing::trace_span!("front message", id = id);

    async fn login_user(ident: CompactString, pwd: CompactString) -> Result<Response, ServerError> {
        let token = user_login(ident, pwd).await?;
        let logined_user = get_user_login(token).await?;
        let resp = Json((token.encode(), logined_user)).into_response();
        Ok(resp)
    }
    fn to_json<T: serde::Serialize>(val: T) -> Result<Response, ServerError> {
        Ok(Response::new(Body::new(
            serde_json::to_string_pretty(&val).map_err(ServerError::into_internal)?,
        )))
    }
    let token = headers
        .get(shared::constant::LOGIN_TOKEN)
        .map(|v| {
            Token::decode(v.as_bytes()).ok_or_else(|| {
                tracing::trace!("{id} deny for bad token");
                ServerError::Fuck
            })
        })
        .transpose()?;
    match message {
        FrontMessage::LoginUser(ident, pwd) => {
            return login_user(ident, pwd).await;
        }
        FrontMessage::Logout => {
            if let Some(token) = token {
                remove_token(token).await?;
            }
            return Ok((Json(()),).into_response());
        }
        _ => {}
    }
    let logined_user = match token {
        Some(token) => {
            let ret = get_user_login(token).await?;
            Some(ret)
        }
        None => None,
    };

    tracing::trace!("id {id} auth {:#?}", logined_user);

    match message {
        FrontMessage::GetProblemFiles(pid) => {
            let Some(user) = logined_user else {
                return Err(ServerError::Fuck);
            };
            if !user.privilege.edit_problems {
                return Err(ServerError::Fuck);
            }
            let files = get_problem(&pid).await.map(|x| x.files)?;
            to_json(&files)
        }
        FrontMessage::GetProblemEditable(pid) => {
            let Some(user) = logined_user else {
                return Err(ServerError::Fuck);
            };
            if !user.privilege.edit_problems {
                return Err(ServerError::Fuck);
            }
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
            let uid = logined_user.map(|x| x.uid).ok_or(ServerError::Fuck)?;
            let rid = submit(uid, submission).await?;
            to_json(rid)
        }
        FrontMessage::RegisterUser(registration) => {
            let uid = user_register(registration).await?;
            to_json(uid)
        }
        FrontMessage::GetLoginedUser => to_json(&logined_user),
        FrontMessage::LoginUser(_, _) | FrontMessage::Logout => unreachable!(),
    }
}
