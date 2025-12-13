mod error;
mod front;
mod judge;
mod problem;
mod record;
mod submission;

use error::EyreResult;
use tokio::net::TcpListener;

use axum::Router;
use axum::routing::{get, post};

pub async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();

    tokio::spawn(judge::track_judge_machines());

    let app = Router::new()
        .route("/", get(front::index))
        .nest(
            "/problem/{pid}",
            Router::new().route("/", get(front::index)),
        )
        .nest("/record/{rid}", Router::new().route("/", get(front::index)))
        .nest("/submit/{pid}", Router::new().route("/", get(front::index)))
        .nest(
            "/assets",
            Router::new().route("/{*path}", get(front::assets)),
        )
        .nest("/wasm", Router::new().route("/{*path}", get(front::wasm)))
        .nest(
            "/api",
            Router::new()
                .nest(
                    "/judge",
                    Router::new().route(
                        "/",
                        get(judge::receive_message).post(judge::receive_message),
                    ),
                )
                .nest(
                    "/front",
                    Router::new().route("/", post(front::receive_front_message)),
                ),
        );

    println!("{:#?}", &app);

    let listener = TcpListener::bind("127.0.0.1:5800").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
