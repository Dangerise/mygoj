use clap::{Parser, Subcommand};
use server::*;
use tokio::net::TcpListener;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug, Clone)]
struct Serve {}

impl Serve {
    async fn serve(&self) {
        tracing::info!("starting...");
        serve::startup().await;
        let app = serve::router();
        let listener = TcpListener::bind("0.0.0.0:5800").await.unwrap();
        tracing::info!("running at 5800..");
        axum::serve(listener, app).await.unwrap();
    }
}

#[derive(Parser, Debug, Clone)]
struct Init {}

impl Init {
    async fn init(&self) {
        init::init_fs(storage_dir()).await.unwrap();
        let db = storage_dir().join("data.db");
        init::init_db(&db).await.unwrap();
        init::init_problems(storage_dir().join("problems")).await.unwrap();
    }
}

#[derive(Debug, Subcommand, Clone)]
enum Command {
    Serve(Serve),
    Init(Init),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("mygoj=trace,server=trace,tower_http::trace=trace")
        .init();
    let cli = Cli::parse();
    match cli.command {
        Command::Serve(args) => {
            args.serve().await;
        }
        Command::Init(args) => {
            args.init().await;
        }
    }
}
