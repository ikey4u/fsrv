use std::{net::SocketAddr, path::PathBuf};

use axum::{routing::post, Router};
use clap::Parser;
use tower_http::services::ServeDir;

#[derive(Debug, Parser)]
struct Cli {
    root: PathBuf,
    #[arg(long, default_value_t = format!("127.0.0.1"))]
    host: String,
    #[arg(long, default_value_t = 80)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let app = Router::new()
        .route(
            "/print",
            post(async |body: String| {
                println!("{body}");
            }),
        )
        .fallback_service(ServeDir::new(&cli.root));
    let addr = format!("{}:{}", cli.host, cli.port);
    let addr = addr
        .parse::<SocketAddr>()
        .expect("failed to parse {addr} as socket address");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!(
        "Serving directory [{}] at [{}]",
        dunce::canonicalize(cli.root)
            .expect("cannot normalize directory")
            .display(),
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
