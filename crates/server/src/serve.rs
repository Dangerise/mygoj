use super::*;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::{any, get, post};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;

pub async fn startup() {
    tokio::spawn(judge::track_judge_machines());
}

pub fn router() -> Router {
    let front = Router::new()
        .route("/assets/{*path}", get(front::assets))
        .route("/wasm/{*path}", get(front::wasm))
        .fallback(front::index);

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let timeout =
        TimeoutLayer::with_status_code(StatusCode::TOO_MANY_REQUESTS, Duration::from_secs(1));

    let api = Router::new()
        .route("/judge", any(judge::receive_message))
        .route("/front", post(front::receive_front_message))
        .route("/front/record_ws", any(record::ws))
        .layer(cors);

    let app = front.nest("/api", api).layer(timeout);

    app
}
