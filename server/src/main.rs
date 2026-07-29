use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

const COLORS: &[&str] = &["#e86a20", "#4488ff", "#ff4444", "#44bb44", "#ffaa22", "#cc66ff", "#22dddd", "#ff66aa"];

type UserId = usize;
type RoomId = String;

struct User {
    tx: mpsc::UnboundedSender<Message>,
    color: String,
}

struct Room {
    users: HashMap<UserId, User>,
    next_id: UserId,
    strokes: Vec<StoredStroke>,
}

#[derive(Clone)]
struct StoredStroke {
    user_id: UserId,
    data: serde_json::Value,
}

type Rooms = Arc<RwLock<HashMap<RoomId, Room>>>;

#[derive(Serialize)]
struct ServerMsg {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<Vec<UserInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strokes: Option<Vec<StrokeEntry>>,
}

#[derive(Clone, Serialize)]
struct StrokeEntry {
    user_id: UserId,
    stroke: serde_json::Value,
}

#[derive(Clone, Serialize)]
struct UserInfo {
    id: UserId,
    color: String,
}

#[derive(Deserialize)]
struct ClientMsg {
    #[serde(rename = "type")]
    msg_type: String,
    room: Option<String>,
    data: Option<serde_json::Value>,
    x: Option<f64>,
    y: Option<f64>,
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, rooms: Rooms) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WS accept error from {addr}: {e}");
            return;
        }
    };

    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Wait for join message to determine room
    let join_msg = match ws_rx.next().await {
        Some(Ok(Message::Text(text))) => text,
        _ => return,
    };

    let parsed: ClientMsg = match serde_json::from_str(&join_msg) {
        Ok(m) => m,
        Err(_) => return,
    };

    if parsed.msg_type != "join" {
        return;
    }

    let room_id = parsed.room.unwrap_or_else(|| "default".to_string());

    // Register user in room
    let (user_id, user_color, user_list, existing_strokes) = {
        let mut rooms = rooms.write().await;
        let room = rooms.entry(room_id.clone()).or_insert(Room {
            users: HashMap::new(),
            next_id: 0,
            strokes: Vec::new(),
        });
        let id = room.next_id;
        room.next_id += 1;
        let color = COLORS[id % COLORS.len()].to_string();
        room.users.insert(
            id,
            User {
                tx: tx.clone(),
                color: color.clone(),
            },
        );
        let user_list: Vec<UserInfo> = room
            .users
            .iter()
            .map(|(uid, u)| UserInfo {
                id: *uid,
                color: u.color.clone(),
            })
            .collect();
        let existing_strokes: Vec<StrokeEntry> = room
            .strokes
            .iter()
            .map(|s| StrokeEntry {
                user_id: s.user_id,
                stroke: s.data.clone(),
            })
            .collect();
        (id, color, user_list, existing_strokes)
    };

    // Send init to the new user
    let init = ServerMsg {
        msg_type: "init".to_string(),
        id: Some(user_id),
        color: Some(user_color.clone()),
        users: Some(user_list.clone()),
        data: None,
        x: None,
        y: None,
        strokes: Some(existing_strokes),
    };
    let _ = ws_tx
        .send(Message::Text(serde_json::to_string(&init).unwrap()))
        .await;

    // Broadcast join to others in room
    let join_broadcast = ServerMsg {
        msg_type: "join".to_string(),
        id: Some(user_id),
        color: Some(user_color.clone()),
        users: None,
        data: None,
        x: None,
        y: None,
        strokes: None,
    };
    broadcast(&rooms, &room_id, user_id, &join_broadcast).await;

    println!("User {user_id} ({user_color}) joined room '{room_id}' [{addr}]");

    // Forward background task: rx → ws_tx
    let forward = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Read loop: ws_rx → broadcast
    while let Some(msg) = ws_rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let parsed: ClientMsg = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                match parsed.msg_type.as_str() {
                    "stroke" | "stroke-start" | "stroke-update" | "shape-update" => {
                        let server_msg = ServerMsg {
                            msg_type: parsed.msg_type.clone(),
                            id: Some(user_id),
                            color: None,
                            users: None,
                            data: parsed.data,
                            x: None,
                            y: None,
                            strokes: None,
                        };
                        broadcast(&rooms, &room_id, user_id, &server_msg).await;
                    }
                    "stroke-end" => {
                        // Store the actual stroke (not the id wrapper) for late joiners
                        if let Some(ref data) = parsed.data {
                            let actual_stroke = data.get("stroke")
                                .and_then(|v| if v.is_null() { None } else { Some(v.clone()) })
                                .unwrap_or_else(|| data.clone());
                            let mut rooms_lock = rooms.write().await;
                            if let Some(room) = rooms_lock.get_mut(&room_id) {
                                room.strokes.push(StoredStroke {
                                    user_id,
                                    data: actual_stroke,
                                });
                            }
                            drop(rooms_lock);
                        }
                        let server_msg = ServerMsg {
                            msg_type: "stroke-end".to_string(),
                            id: Some(user_id),
                            color: None,
                            users: None,
                            data: parsed.data,
                            x: None,
                            y: None,
                            strokes: None,
                        };
                        broadcast(&rooms, &room_id, user_id, &server_msg).await;
                    }
                    "cursor" => {
                        let server_msg = ServerMsg {
                            msg_type: "cursor".to_string(),
                            id: Some(user_id),
                            color: Some(user_color.clone()),
                            users: None,
                            data: None,
                            x: parsed.x,
                            y: parsed.y,
                            strokes: None,
                        };
                        broadcast(&rooms, &room_id, user_id, &server_msg).await;
                    }
                    _ => {}
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    // Cleanup on disconnect
    forward.abort();
    {
        let mut rooms = rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.users.remove(&user_id);
            room.strokes.retain(|s| s.user_id != user_id);
            if room.users.is_empty() {
                rooms.remove(&room_id);
                println!("Room '{room_id}' deleted (empty)");
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
    };
    broadcast(&rooms, &room_id, user_id, &leave_msg).await;
    println!("User {user_id} left room '{room_id}'");
}

async fn broadcast(rooms: &Rooms, room_id: &str, sender_id: UserId, msg: &ServerMsg) {
    let text = serde_json::to_string(msg).unwrap();
    let rooms_lock = rooms.read().await;
    if let Some(room) = rooms_lock.get(room_id) {
        for (&uid, user) in &room.users {
            if uid != sender_id {
                let _ = user.tx.send(Message::Text(text.clone()));
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:9876";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("scrib-server listening on ws://{addr}");

    let rooms: Rooms = Arc::new(RwLock::new(HashMap::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        let rooms = rooms.clone();
        tokio::spawn(handle_connection(stream, addr, rooms));
    }
}
