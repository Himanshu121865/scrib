mod room;
mod ws;

use std::sync::Arc;

use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
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

    #[arg(long, default_value = "data")]
    data_dir: String,

    #[arg(long, default_value_t = 50)]
    max_users: usize,

    #[arg(long, default_value_t = 30)]
    save_interval: u64,
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

    tokio::fs::create_dir_all(&args.data_dir)
        .await
        .expect("Failed to create data directory");

    let save_interval = args.save_interval;
    let state = Arc::new(AppState {
        rooms: Arc::new(RwLock::new(room::RoomMap::new())),
        data_dir: args.data_dir.into(),
        max_users: args.max_users,
    });

    // --- periodic background save + stats ---
    let save_state = state.clone();
    let save_handle = tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(save_interval));
        loop {
            tick.tick().await;
            let mut map = save_state.rooms.write().await;
            let room_count = map.len();
            let mut total_users = 0;
            let mut total_strokes = 0;
            let mut saved = 0;
            for (room_id, room) in map.iter_mut() {
                total_users += room.users.len();
                total_strokes += room.strokes.len();
                let path = room::room_path(&save_state.data_dir, room_id);
                if room.dirty {
                    room.save(&path).await;
                    saved += 1;
                }
            }
            if saved > 0 || room_count > 0 {
                info!("stats: {room_count} room(s), {total_users} user(s), {total_strokes} stroke(s), {saved} saved");
            }
        }
    });

    // --- shutdown signal ---
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("shutting down...");
        let _ = shutdown_tx.send(());
    });

    // --- accept loop ---
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

    // --- graceful shutdown ---
    save_handle.abort();

    let mut saved = 0;
    {
        let mut map = state.rooms.write().await;
        let paths: Vec<_> = map.keys().cloned().collect();
        for room_id in &paths {
            if let Some(room) = map.get_mut(room_id) {
                if room.dirty {
                    let path = room::room_path(&state.data_dir, room_id);
                    room.save(&path).await;
                    saved += 1;
                }
            }
        }
    }
    if saved > 0 {
        info!("saved {saved} room(s) on shutdown");
    }

    info!("server stopped");
}
