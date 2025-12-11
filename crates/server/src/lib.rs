mod judge;
mod problem;
mod record;
mod submission;

use rust_embed::RustEmbed;
use salvo::prelude::*;
use salvo::serve_static::static_embed;
use std::sync::LazyLock;

#[cfg(debug_assertions)]
#[derive(RustEmbed)]
#[folder = "../../target/dx/front/debug/web/public"]
struct Front;

#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "../../target/dx/front/release/web/public"]
struct Front;

static INDEX: LazyLock<String> = LazyLock::new(|| {
    let data = Front::get("index.html").unwrap().data;
    let html = String::from_utf8_lossy(&data);
    html.into_owned()
});

#[handler]
async fn index(resp: &mut Response) {
    let file: &str = INDEX.as_str();
    resp.render(Text::Html(file));
}

pub async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();

    tokio::spawn(judge::check_alive());

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;

    let router = Router::new()
        .get(index)
        .push(Router::with_path("problem/{*id}").get(index))
        .push(Router::with_path("submit/{*id}").get(index))
        .push(Router::with_path("record/{*id}").get(index))
        .push(Router::with_path("judge-status").get(index))
        .push(
            Router::with_path("api")
                .push(Router::with_path("problem_front").get(problem::problem_front))
                .push(Router::with_path("submit").post(submission::receive_submission))
                .push(Router::with_path("record").get(record::get_record))
                .push(Router::with_path("judge_machines").get(judge::judge_machines))
                .push(
                    Router::with_path("judge")
                        .push(Router::with_path("connect").post(judge::connect)),
                ),
        )
        .push(Router::with_path("{*path}").get(static_embed::<Front>()));

    dbg!(Front::iter().collect::<Vec<_>>());
    dbg!(&router);

    println!("{:?}", router);

    Server::new(acceptor).serve(router).await;

    Ok(())
}
