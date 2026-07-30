use std::time::Instant;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::mpsc::error::TrySendError;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

use crate::room::{self, ServerMsg, SharedState, UserId};

#[derive(Deserialize)]
struct ClientMsg {
    #[serde(rename = "type")]
    msg_type: String,
    room: Option<String>,
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

pub async fn handle_connection(stream: TcpStream, addr: std::net::SocketAddr, state: SharedState) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!("WS accept error from {addr}: {e}");
            return;
        }
    };

    let (mut ws_tx, mut ws_rx) = ws.split();

    // --- join handshake (with timeout) ---
    let join_msg = match timeout(Duration::from_secs(room::JOIN_TIMEOUT_SECS), ws_rx.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => text,
        Ok(_) => return,
        Err(_) => {
            let _ = ws_tx.send(err_msg("join timeout")).await;
            return;
        }
    };

    // --- validation (uses ws_tx directly before forward takes over) ---
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
    let room_id = parsed.room.unwrap_or_else(|| "default".to_string());
    if !room::validate_room_id(&room_id) {
        let _ = ws_tx
            .send(err_msg(&format!("invalid room id: '{room_id}'")))
            .await;
        return;
    }

    let path = room::room_path(&state.data_dir, &room_id);

    // --- set up outgoing channel (forward task takes over ws_tx) ---
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(room::MSG_CHANNEL_CAP);
    let init_tx = tx.clone();

    let forward = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // --- join room (atomically check capacity + add user) ---
    let join_result = {
        let mut map = state.rooms.write().await;
        let room = map.entry(room_id.clone()).or_insert_with(|| {
            let p = path.clone();
            room::Room::load(&p).unwrap_or_else(|_| {
                info!("created new room '{room_id}'");
                room::Room::new()
            })
        });

        if room.users.len() >= state.max_users {
            Err(())
        } else {
            let (uid, color) = room.add_user(tx);
            assign_stroke_ids(room);
            let init = ServerMsg {
                msg_type: "init".to_string(),
                id: Some(uid),
                color: Some(color.clone()),
                users: Some(room.user_list()),
                strokes: Some(room.stroke_entries()),
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
            let _ = init_tx.try_send(err_msg("room is full")).ok();
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
    let _ = init_tx.send(Message::Text(init_text)).await.ok();

    // --- broadcast join to other users ---
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
    broadcast(&state, &room_id, user_id, &join_broadcast).await;
    info!("user {user_id} ({user_color}) joined room '{room_id}' [{addr}]");

    // --- message loop ---
    let mut limiter = MsgRateLimiter::new();
    loop {
        let msg = match ws_rx.next().await {
            Some(Ok(Message::Text(t))) => t,
            Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => continue,
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
            _ => continue,
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
                broadcast(&state, &room_id, user_id, &msg).await;
            }
            "stroke-end" => {
                handle_stroke_end(&state, &room_id, user_id, parsed).await;
            }
            "clear" => {
                handle_clear(&state, &room_id).await;
            }
            "erase" => {
                handle_erase(&state, &room_id, user_id, parsed).await;
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
                broadcast(&state, &room_id, user_id, &msg).await;
            }
            _ => {}
        }
    }

    forward.abort();
    disconnect_user(&state, &room_id, user_id).await;
    info!("user {user_id} left room '{room_id}'");
}

async fn handle_stroke_end(state: &SharedState, room_id: &str, user_id: UserId, parsed: ClientMsg) {
    let stored_data = parsed.data.as_ref().and_then(|d| {
        d.get("stroke")
            .filter(|v| !v.is_null())
            .cloned()
            .or(Some(d.clone()))
    });

    let path = room::room_path(&state.data_dir, room_id);

    if let Some(mut stroke) = stored_data {
        ensure_id(&mut stroke, user_id);
        let mut map = state.rooms.write().await;
        if let Some(room) = map.get_mut(room_id) {
            room.add_stroke(user_id, stroke);
            room.save(&path).await;
        }
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
    broadcast(state, room_id, user_id, &msg).await;
}

async fn handle_clear(state: &SharedState, room_id: &str) {
    let path = room::room_path(&state.data_dir, room_id);
    let mut map = state.rooms.write().await;
    if let Some(room) = map.get_mut(room_id) {
        room.clear_strokes();
        room.save(&path).await;
    }
}

async fn handle_erase(state: &SharedState, room_id: &str, user_id: UserId, parsed: ClientMsg) {
    let owners = if let Some(ref ids) = parsed.ids {
        let path = room::room_path(&state.data_dir, room_id);
        let mut map = state.rooms.write().await;
        if let Some(room) = map.get_mut(room_id) {
            let owners = room.remove_strokes(ids, user_id);
            if !owners.is_empty() {
                room.save(&path).await;
            }
            owners
        } else {
            Vec::new()
        }
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
    broadcast(state, room_id, user_id, &msg).await;
}

async fn disconnect_user(state: &SharedState, room_id: &str, user_id: UserId) {
    let path = room::room_path(&state.data_dir, room_id);
    {
        let mut map = state.rooms.write().await;
        if let Some(room) = map.get_mut(room_id) {
            room.users.remove(&user_id);
            if room.users.is_empty() {
                if room.dirty {
                    room.save(&path).await;
                }
                map.remove(room_id);
                info!("room '{room_id}' deleted (empty)");
            } else if room.dirty {
                room.save(&path).await;
            }
        }
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
    broadcast(state, room_id, user_id, &leave_msg).await;
}

async fn broadcast(state: &SharedState, room_id: &str, sender_id: UserId, msg: &ServerMsg) {
    let text = match serde_json::to_string(msg) {
        Ok(t) => t,
        Err(e) => {
            warn!("failed to serialize broadcast: {e}");
            return;
        }
    };
    let map = state.rooms.read().await;
    if let Some(room) = map.get(room_id) {
        for (&uid, user) in &room.users {
            if uid == sender_id {
                continue;
            }
            match user.tx.try_send(Message::Text(text.clone())) {
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

fn assign_stroke_ids(room: &mut room::Room) {
    let mut changed = false;
    for (idx, stored) in room.strokes.iter_mut().enumerate() {
        if let serde_json::Value::Object(ref mut map) = stored.data {
            if !map.contains_key("id") {
                let sid = format!("srv_{}_{}", stored.user_id, idx);
                map.insert("id".to_string(), serde_json::Value::String(sid));
                changed = true;
            }
        }
    }
    if changed {
        room.dirty = true;
    }
}

fn err_msg(text: &str) -> Message {
    Message::Text(json!({"type": "error", "message": text}).to_string())
}
