mod dbg;
mod front;
mod judge;
mod problem;
mod record;
mod submission;
mod user;

use tokio::net::TcpListener;
use shared::error::ServerError;

use axum::Router;
use axum::routing::{get, post};

pub async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();

    tokio::spawn(judge::track_judge_machines());

    dbg::dbg().await;

    let front = Router::new()
        .route("/assets/{*path}", get(front::assets))
        .route("/wasm/{*path}", get(front::wasm))
        .fallback(front::index);

    let api = Router::new()
        .route(
            "/judge",
            get(judge::receive_message).post(judge::receive_message),
        )
        .route("/front", post(front::receive_front_message));

    let app = front.nest("/api", api);

    println!("{:#?}", &app);

    let listener = TcpListener::bind("127.0.0.1:5800").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
