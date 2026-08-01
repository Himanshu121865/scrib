mod room;
mod ws;

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::EnvFilter;
use warp::Filter;

use crate::room::AppState;

#[derive(Parser)]
#[command(name = "scrib-server", version, about = "Multiplayer drawing server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    addr: String,

    #[arg(long)]
    port: Option<u16>,

    #[arg(long, default_value_t = 50)]
    max_users: usize,
}

fn default_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9876)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let args = Args::parse();
    let port = args.port.unwrap_or_else(default_port);
    let bind: SocketAddr = format!("{}:{}", args.addr, port)
        .parse()
        .expect("invalid bind address");

    let state = Arc::new(AppState {
        board: Arc::new(RwLock::new(room::Board::new())),
        max_users: args.max_users,
    });

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(with_state(state))
        .map(
            |ws: warp::ws::Ws, addr: Option<SocketAddr>, state: Arc<AppState>| {
                ws.on_upgrade(move |socket| ws::handle_ws(socket, addr, state))
            },
        );

    let static_route = warp::get().and(warp::fs::dir("www"));

    info!("listening on http://{bind}");
    warp::serve(ws_route.or(static_route)).run(bind).await;
}

fn with_state(
    state: Arc<AppState>,
) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}
