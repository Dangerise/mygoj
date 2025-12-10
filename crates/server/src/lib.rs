mod problem;

use eyre::eyre;
use rust_embed::RustEmbed;
use salvo::prelude::*;
use salvo::serve_static::static_embed;
use std::sync::LazyLock;
use tokio::fs;

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

#[handler]
async fn problem_front(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let pid = req
        .query::<String>("pid")
        .ok_or_else(|| eyre!("pid nod found"))?;

    let path = dirs::home_dir()
        .unwrap()
        .join("mygoj")
        .join("problems")
        .join(&pid)
        .join("config.json");

    tracing::info!("read problem config file {}", path.display());

    let config = fs::read_to_string(&path).await?;
    let mut problem: problem::Problem = serde_json::from_str(&config)?;
    problem.front.pid = pid.clone();

    tracing::info!("response problem front {:?}", &problem.front);

    resp.render(Json(problem.front));
    Ok(())
}

pub async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;

    let router = Router::new()
        .get(index)
        .push(Router::with_path("problem/{*id}").get(index))
        .push(Router::with_path("api").push(Router::with_path("problem_front").get(problem_front)))
        .push(Router::with_path("{*path}").get(static_embed::<Front>()));

    dbg!(Front::iter().collect::<Vec<_>>());
    dbg!(&router);

    println!("{:?}", router);

    Server::new(acceptor).serve(router).await;

    Ok(())
}
