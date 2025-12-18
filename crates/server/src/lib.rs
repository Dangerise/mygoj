mod dbg;
mod front;
mod judge;
mod problem;
mod record;
mod user;

use shared::error::ServerError;
use tokio::net::TcpListener;

use axum::Router;
use axum::routing::{any, get, post};
use tower_http::cors::CorsLayer;

pub async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();

    tokio::spawn(judge::track_judge_machines());

    dbg::dbg().await;

    let front = Router::new()
        .route("/assets/{*path}", get(front::assets))
        .route("/wasm/{*path}", get(front::wasm))
        .fallback(front::index);

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let api = Router::new()
        .route("/judge", any(judge::receive_message))
        .route("/front", post(front::receive_front_message))
        .route("/front/record_ws", any(record::ws))
        .layer(cors);

    let app = front.nest("/api", api);

    println!("{:#?}", &app);

    let listener = TcpListener::bind("0.0.0.0:5800").await.unwrap();

    tracing::info!("running");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
