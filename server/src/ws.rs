use std::time::Instant;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc::error::TrySendError;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};
use warp::ws::Message;

use crate::room::{self, ServerMsg, SharedState, UserId};

#[derive(Deserialize)]
struct ClientMsg {
    #[serde(rename = "type")]
    msg_type: String,
    data: Option<serde_json::Value>,
    x: Option<f64>,
    y: Option<f64>,
    ids: Option<Vec<String>>,
}

struct MsgRateLimiter {
    count: u32,
    window_start: Instant,
}

impl MsgRateLimiter {
    fn new() -> Self {
        MsgRateLimiter {
            count: 0,
            window_start: Instant::now(),
        }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();
        if now - self.window_start > Duration::from_secs(1) {
            self.count = 0;
            self.window_start = now;
        }
        self.count += 1;
        self.count <= room::MSG_RATE_WINDOW
    }
}

pub async fn handle_ws(
    ws: warp::ws::WebSocket,
    addr: Option<std::net::SocketAddr>,
    state: SharedState,
) {
    let addr_str = addr.map_or_else(|| "unknown".to_string(), |a| a.to_string());

    let (mut ws_tx, mut ws_rx) = ws.split();

    let join_msg = match timeout(Duration::from_secs(room::JOIN_TIMEOUT_SECS), ws_rx.next()).await {
        Ok(Some(Ok(msg))) => match msg.to_str() {
            Ok(t) => t.to_string(),
            Err(_) => return,
        },
        Ok(_) => return,
        Err(_) => {
            let _ = ws_tx.send(err_msg("join timeout")).await;
            return;
        }
    };

    if join_msg.len() > room::MAX_MSG_BYTES {
        let _ = ws_tx.send(err_msg("message too large")).await;
        return;
    }
    let parsed: ClientMsg = match serde_json::from_str(&join_msg) {
        Ok(m) => m,
        Err(e) => {
            let _ = ws_tx.send(err_msg(&format!("invalid JSON: {e}"))).await;
            return;
        }
    };
    if parsed.msg_type != "join" {
        let _ = ws_tx.send(err_msg("first message must be join")).await;
        return;
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(room::MSG_CHANNEL_CAP);
    let init_tx = tx.clone();

    let forward = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let join_result = {
        let mut board = state.board.write().await;
        if board.users.len() >= state.max_users {
            Err(())
        } else {
            let (uid, color) = board.add_user(tx);
            let init = ServerMsg {
                msg_type: "init".to_string(),
                id: Some(uid),
                color: Some(color.clone()),
                users: Some(board.user_list()),
                strokes: Some(board.stroke_entries()),
                data: None,
                x: None,
                y: None,
                ids: None,
                owners: None,
            };
            Ok((uid, color, init))
        }
    };

    let (user_id, user_color, init) = match join_result {
        Ok(triple) => triple,
        Err(_) => {
            let _ = init_tx.try_send(err_msg("board is full")).ok();
            forward.abort();
            return;
        }
    };

    let init_text = match serde_json::to_string(&init) {
        Ok(t) => t,
        Err(e) => {
            warn!("failed to serialize init: {e}");
            forward.abort();
            return;
        }
    };
    let _ = init_tx.send(Message::text(init_text)).await.ok();

    let join_broadcast = ServerMsg {
        msg_type: "join".to_string(),
        id: Some(user_id),
        color: Some(user_color.clone()),
        users: None,
        data: None,
        x: None,
        y: None,
        strokes: None,
        ids: None,
        owners: None,
    };
    broadcast(&state, user_id, &join_broadcast).await;
    info!("user {user_id} ({user_color}) connected [{addr_str}]");

    let mut limiter = MsgRateLimiter::new();
    loop {
        let msg = match ws_rx.next().await {
            Some(Ok(m)) if m.is_text() => match m.to_str() {
                Ok(t) => t.to_string(),
                Err(_) => continue,
            },
            Some(Ok(m)) if m.is_close() => break,
            Some(Ok(_)) => continue,
            Some(Err(_)) | None => break,
        };

        if msg.len() > room::MAX_MSG_BYTES {
            warn!("oversize message from user {user_id}");
            continue;
        }

        let parsed: ClientMsg = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !limiter.allow() {
            warn!("rate limit exceeded for user {user_id}");
            continue;
        }

        match parsed.msg_type.as_str() {
            "stroke" | "stroke-start" | "stroke-update" | "shape-update" => {
                let msg = ServerMsg {
                    msg_type: parsed.msg_type.clone(),
                    id: Some(user_id),
                    color: None,
                    users: None,
                    data: parsed.data,
                    x: None,
                    y: None,
                    strokes: None,
                    ids: None,
                    owners: None,
                };
                broadcast(&state, user_id, &msg).await;
            }
            "stroke-end" => {
                handle_stroke_end(&state, user_id, parsed).await;
            }
            "clear" => {
                handle_clear(&state).await;
            }
            "erase" => {
                handle_erase(&state, user_id, parsed).await;
            }
            "cursor" => {
                let msg = ServerMsg {
                    msg_type: "cursor".to_string(),
                    id: Some(user_id),
                    color: Some(user_color.clone()),
                    users: None,
                    data: None,
                    x: parsed.x,
                    y: parsed.y,
                    strokes: None,
                    ids: None,
                    owners: None,
                };
                broadcast(&state, user_id, &msg).await;
            }
            _ => {}
        }
    }

    forward.abort();
    disconnect_user(&state, user_id).await;
    info!("user {user_id} disconnected");
}

async fn handle_stroke_end(state: &SharedState, user_id: UserId, parsed: ClientMsg) {
    let stored_data = parsed.data.as_ref().and_then(|d| {
        d.get("stroke")
            .filter(|v| !v.is_null())
            .cloned()
            .or(Some(d.clone()))
    });

    if let Some(mut stroke) = stored_data {
        ensure_id(&mut stroke, user_id);
        let mut board = state.board.write().await;
        board.add_stroke(user_id, stroke);
    }

    let msg = ServerMsg {
        msg_type: "stroke-end".to_string(),
        id: Some(user_id),
        color: None,
        users: None,
        data: parsed.data,
        x: None,
        y: None,
        strokes: None,
        ids: None,
        owners: None,
    };
    broadcast(state, user_id, &msg).await;
}

async fn handle_clear(state: &SharedState) {
    let mut board = state.board.write().await;
    board.clear_strokes();
}

async fn handle_erase(state: &SharedState, user_id: UserId, parsed: ClientMsg) {
    let owners = if let Some(ref ids) = parsed.ids {
        let mut board = state.board.write().await;
        board.remove_strokes(ids, user_id)
    } else {
        Vec::new()
    };

    let msg = ServerMsg {
        msg_type: "erase".to_string(),
        id: Some(user_id),
        color: None,
        users: None,
        data: None,
        x: None,
        y: None,
        strokes: None,
        ids: parsed.ids.clone(),
        owners: if owners.is_empty() {
            None
        } else {
            Some(owners)
        },
    };
    broadcast(state, user_id, &msg).await;
}

async fn disconnect_user(state: &SharedState, user_id: UserId) {
    {
        let mut board = state.board.write().await;
        board.users.remove(&user_id);
    }

    let leave_msg = ServerMsg {
        msg_type: "leave".to_string(),
        id: Some(user_id),
        color: None,
        users: None,
        data: None,
        x: None,
        y: None,
        strokes: None,
        ids: None,
        owners: None,
    };
    broadcast(state, user_id, &leave_msg).await;
}

async fn broadcast(state: &SharedState, sender_id: UserId, msg: &ServerMsg) {
    let text = match serde_json::to_string(msg) {
        Ok(t) => t,
        Err(e) => {
            warn!("failed to serialize broadcast: {e}");
            return;
        }
    };
    let board = state.board.read().await;
    for (&uid, user) in &board.users {
        if uid == sender_id {
            continue;
        }
        match user.tx.try_send(Message::text(text.clone())) {
            Ok(_) => {}
            Err(TrySendError::Full(_)) => {
                if msg.msg_type != "cursor" {
                    warn!(
                        "dropping message for slow user {uid} (type={})",
                        msg.msg_type
                    );
                }
            }
            Err(TrySendError::Closed(_)) => {}
        }
    }
}

fn ensure_id(stroke: &mut serde_json::Value, user_id: UserId) {
    if let serde_json::Value::Object(ref mut map) = stroke {
        if !map.contains_key("id") {
            let sid = format!(
                "srv_{}_{}",
                user_id,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            map.insert("id".to_string(), serde_json::Value::String(sid));
        }
    }
}

fn err_msg(text: &str) -> Message {
    Message::text(json!({"type": "error", "message": text}).to_string())
}
