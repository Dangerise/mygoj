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

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;

    let router = Router::new()
        .get(index)
        .push(Router::with_path("blog/{*id}").get(index))
        .push(Router::with_path("{*path}").get(static_embed::<Front>()));

    dbg!(Front::iter().collect::<Vec<_>>());
    dbg!(&router);

    println!("{:?}", router);

    Server::new(acceptor).serve(router).await;

    Ok(())
}
