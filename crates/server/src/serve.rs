use super::*;
use axum::Router;
use axum::http::{HeaderName, StatusCode, header::*};
use axum::routing::{any, get};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub async fn startup() {
    let path = storage_dir().join("data.db");
    let path = path.as_os_str().to_str().unwrap();
    db::database_connect(path).await.unwrap();
    judge::init_queue().await.unwrap();
    tokio::spawn(judge::track_judge_machines());
}

pub fn router() -> Router {
    let front = Router::new()
        .route("/assets/{*path}", get(front::assets))
        .route("/wasm/{*path}", get(front::wasm))
        .fallback(front::index);

    let x_request_id = HeaderName::from_static("x-request-id");
    let set_id = SetRequestIdLayer::new(x_request_id, MakeRequestUuid);

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::AllowHeaders::list([
            AUTHORIZATION,
            CONTENT_TYPE,
        ]));

    let timeout =
        TimeoutLayer::with_status_code(StatusCode::TOO_MANY_REQUESTS, Duration::from_secs(1));

    let trace = TraceLayer::new_for_http().make_span_with(
        DefaultMakeSpan::new()
            .level(tracing::Level::TRACE)
            .include_headers(true),
    );

    let front_api = Router::new()
        .route("/", any(front::receive_front_message))
        .route("/record_ws", any(record::ws))
        .route(
            "/commit_problem_files/{pid}",
            any(problem::commit_problem_files),
        )
        .layer(axum::middleware::from_fn(front::logined_user_layer))
        .route("/login", any(front::login))
        .route("/logout", any(front::logout))
        .layer(trace);

    let api = Router::new()
        .route("/judge", any(judge::receive_message))
        .nest("/front", front_api)
        .layer(cors);

    front.nest("/api", api).layer(set_id).layer(timeout)
}
