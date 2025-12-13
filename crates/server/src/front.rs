use super::EyreResult;
use super::judge::judge_machines;
use super::problem::get_problem_front;
use super::record::get_record;
use super::submission::receive_submission;
use rust_embed::RustEmbed;
use shared::front::FrontMessage;
use std::borrow::Cow;
use std::ffi::OsStr;
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
    tracing::info!("assets {path}");
    dir(format!("assets/{path}")).await
}

pub async fn wasm(Path(path): Path<String>) -> Result<Response, StatusCode> {
    tracing::info!("wasm {path}");
    dir(format!("wasm/{path}")).await
}

pub async fn receive_front_message(Json(message): Json<FrontMessage>) -> EyreResult<String> {
    match message {
        FrontMessage::GetProblemFront(pid) => {
            let front = get_problem_front(&pid).await?;
            Ok(serde_json::to_string_pretty(&front)?)
        }
        FrontMessage::CheckJudgeMachines => {
            let res = judge_machines().await?;
            Ok(serde_json::to_string_pretty(&res)?)
        }
        FrontMessage::GetRecord(rid) => {
            let rec = get_record(rid).await?;
            Ok(serde_json::to_string_pretty(&rec)?)
        }
        FrontMessage::Submit(submission) => {
            let rid = receive_submission(submission).await?;
            Ok(serde_json::to_string_pretty(&rid)?)
        }
    }
}
