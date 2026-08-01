mod room;
mod ws;

use std::sync::Arc;

use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::room::AppState;

#[derive(Parser)]
#[command(name = "scrib-server", version, about = "Multiplayer drawing server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    addr: String,

    #[arg(long, default_value_t = 9876)]
    port: u16,

    #[arg(long, default_value_t = 50)]
    max_users: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let args = Args::parse();
    let bind = format!("{}:{}", args.addr, args.port);
    let listener = TcpListener::bind(&bind).await.expect("Failed to bind");
    info!("listening on ws://{bind}");

    let state = Arc::new(AppState {
        board: Arc::new(RwLock::new(room::Board::new())),
        max_users: args.max_users,
    });

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("shutting down...");
        let _ = shutdown_tx.send(());
    });

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        let state = state.clone();
                        tokio::spawn(ws::handle_connection(stream, addr, state));
                    }
                    Err(e) => warn!("accept error: {e}"),
                }
            }
            _ = &mut shutdown_rx => break,
        }
    }

    info!("server stopped");
}
