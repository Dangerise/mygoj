use super::ServerError;
use super::judge::judge_machines;
use super::problem::get_problem_front;
use super::record::get_record;
use super::submission::receive_submission;
use super::user::{get_user_login, register_user, user_login};
use compact_str::CompactString;
use rust_embed::RustEmbed;
use shared::front::FrontMessage;
use shared::token::Token;
use std::borrow::Cow;
use std::sync::LazyLock;

use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;

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

pub async fn receive_front_message(
    jar: CookieJar,
    Json(message): Json<FrontMessage>,
) -> Result<Response, ServerError> {
    async fn login_user(email: CompactString, pwd: CompactString) -> Result<Response, ServerError> {
        let token = user_login(email, pwd).await?;
        let logined_user = get_user_login(token).await?;
        let cookie = Cookie::new(shared::cookies::LOGIN_STATE, token.encode());
        let jar = CookieJar::new().add(cookie);
        let resp = (jar, Json(logined_user)).into_response();
        Ok(resp)
    }
    fn to_json<T: serde::Serialize>(val: T) -> Result<Response, ServerError> {
        Ok(Response::new(Body::new(
            serde_json::to_string_pretty(&val).map_err(ServerError::into_internal)?,
        )))
    }
    match message {
        FrontMessage::LoginUser(email, pwd) => {
            return login_user(email, pwd).await;
        }
        FrontMessage::Logout => {
            return Ok((
                CookieJar::new().add(
                    Cookie::build(shared::cookies::LOGIN_STATE)
                        .removal()
                        .build(),
                ),
                Json(()),
            )
                .into_response());
        }
        _ => {}
    }
    let logined_user = match jar.get(shared::cookies::LOGIN_STATE) {
        Some(v) => {
            let token = Token::decode(v.value().as_ref()).ok_or(ServerError::Fuck)?;
            let ret = get_user_login(token).await?;
            Some(ret)
        }
        None => None,
    };
    match message {
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
            let rid = receive_submission(submission).await?;
            to_json(&rid)
        }
        FrontMessage::RegisterUser(registration) => {
            let uid = register_user(registration).await?;
            to_json(&uid)
        }
        FrontMessage::GetLoginedUser => to_json(&logined_user),
        FrontMessage::LoginUser(_, _) | FrontMessage::Logout => unreachable!(),
    }
}
