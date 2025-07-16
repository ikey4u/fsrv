use std::{
    net::SocketAddr,
    path::{PathBuf, MAIN_SEPARATOR},
    sync::Arc,
};

use axum::{extract::State, routing::post, Json, Router};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
struct Cli {
    root: PathBuf,
    #[arg(long, default_value_t = format!("127.0.0.1"))]
    host: String,
    #[arg(long, default_value_t = 80)]
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintReq {
    pub desc: String,
    pub msg: String,
}

pub struct AppState {
    pub srvdir: PathBuf,
}

pub async fn printer(state: State<Arc<AppState>>, body: Json<PrintReq>) {
    let desc = &body.desc;
    if body.msg.trim().is_empty() {
        println!("{desc}");
    } else {
        println!("{desc}\n{}", body.msg);
    }

    if ![".txt", ".log", ".json"].iter().any(|x| desc.ends_with(x)) {
        return;
    }
    let Some(filename) = desc.split(MAIN_SEPARATOR).next_back() else {
        println!(
            "file name is not found in {desc}, write msg to file is disabled"
        );
        return;
    };
    let Ok(path) =
        std::path::absolute(state.srvdir.join(format!("_{filename}")))
    else {
        println!(
            "failed to canonicalize path for file {desc}, write is disabled"
        );
        return;
    };
    match std::fs::write(&path, body.msg.as_bytes()) {
        Ok(_) => {
            println!("write to {} done", path.display());
        }
        Err(e) => {
            println!(
                "failed to write body msg to file {}: {e:?}",
                path.display()
            );
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()
                .unwrap(),
        )
        .init();

    let cli = Cli::parse();
    let state = std::sync::Arc::new(AppState {
        srvdir: cli.root.clone(),
    });
    let app = Router::new()
        .route("/print", post(printer))
        .with_state(state)
        .fallback_service(ServeDir::new(&cli.root));
    let app = app.layer(TraceLayer::new_for_http());
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
