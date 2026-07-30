use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;

pub type UserId = usize;
pub type RoomId = String;

pub const MAX_STROKES_PER_ROOM: usize = 500;
pub const MAX_ROOM_ID_LEN: usize = 64;
pub const MAX_MSG_BYTES: usize = 256 * 1024;
pub const MSG_RATE_WINDOW: u32 = 120;
pub const MSG_CHANNEL_CAP: usize = 128;
pub const JOIN_TIMEOUT_SECS: u64 = 15;
pub const COLORS: &[&str] = &[
    "#e86a20", "#4488ff", "#ff4444", "#44bb44", "#ffaa22", "#cc66ff", "#22dddd", "#ff66aa",
];

pub struct User {
    pub tx: mpsc::Sender<Message>,
    pub color: String,
}

pub struct Room {
    pub users: HashMap<UserId, User>,
    pub next_id: UserId,
    pub strokes: Vec<StoredStroke>,
    pub dirty: bool,
}

impl Room {
    pub fn new() -> Self {
        Room {
            users: HashMap::new(),
            next_id: 0,
            strokes: Vec::with_capacity(64),
            dirty: false,
        }
    }

    pub fn add_user(&mut self, tx: mpsc::Sender<Message>) -> (UserId, String) {
        let id = self.next_id;
        self.next_id += 1;
        let color = COLORS[id % COLORS.len()].to_string();
        self.users.insert(
            id,
            User {
                tx,
                color: color.clone(),
            },
        );
        (id, color)
    }

    pub fn add_stroke(&mut self, user_id: UserId, data: serde_json::Value) {
        if self.strokes.len() >= MAX_STROKES_PER_ROOM {
            self.strokes.remove(0);
        }
        self.strokes.push(StoredStroke { user_id, data });
        self.dirty = true;
    }

    pub fn clear_strokes(&mut self) {
        self.strokes.clear();
        self.dirty = true;
    }

    pub fn remove_strokes(&mut self, ids: &[String], user_id: UserId) -> Vec<UserId> {
        let mut owners = Vec::new();
        self.strokes.retain(|s| {
            let sid = s.data.get("id").and_then(|v| v.as_str());
            let matched = s.user_id == user_id && ids.iter().any(|id| Some(id.as_str()) == sid);
            if matched {
                owners.push(s.user_id);
            }
            !matched
        });
        if !owners.is_empty() {
            self.dirty = true;
        }
        owners
    }

    pub fn user_list(&self) -> Vec<UserInfo> {
        self.users
            .iter()
            .map(|(uid, u)| UserInfo {
                id: *uid,
                color: u.color.clone(),
            })
            .collect()
    }

    pub fn stroke_entries(&self) -> Vec<StrokeEntry> {
        self.strokes
            .iter()
            .map(|s| StrokeEntry {
                user_id: s.user_id,
                stroke: s.data.clone(),
            })
            .collect()
    }

    pub async fn save(&mut self, path: &Path) {
        if !self.dirty {
            return;
        }
        let data = RoomData {
            next_id: self.next_id,
            strokes: self.strokes.clone(),
        };
        let json = match serde_json::to_string(&data) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("failed to serialize room data: {e}");
                return;
            }
        };
        let tmp = path.with_extension("json.tmp");
        match tokio::fs::write(&tmp, &json).await {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("failed to write temp file: {e}");
                return;
            }
        }
        match tokio::fs::rename(&tmp, path).await {
            Ok(_) => self.dirty = false,
            Err(e) => {
                tracing::warn!("failed to rename temp file: {e}");
            }
        }
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let data: RoomData = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Room {
            users: HashMap::new(),
            next_id: data.next_id,
            strokes: data.strokes,
            dirty: false,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredStroke {
    pub user_id: UserId,
    pub data: serde_json::Value,
}

#[derive(Clone, Serialize)]
pub struct StrokeEntry {
    pub user_id: UserId,
    pub stroke: serde_json::Value,
}

#[derive(Clone, Serialize)]
pub struct UserInfo {
    pub id: UserId,
    pub color: String,
}

#[derive(Serialize, Deserialize)]
struct RoomData {
    next_id: UserId,
    strokes: Vec<StoredStroke>,
}

pub type RoomMap = HashMap<RoomId, Room>;
pub type Rooms = Arc<RwLock<RoomMap>>;

pub struct AppState {
    pub rooms: Rooms,
    pub data_dir: PathBuf,
    pub max_users: usize,
}

pub type SharedState = Arc<AppState>;

pub fn validate_room_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ROOM_ID_LEN
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

pub fn room_path(data_dir: &Path, room_id: &str) -> PathBuf {
    data_dir.join(format!("{room_id}.json"))
}

#[derive(Serialize)]
pub struct ServerMsg {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<UserInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strokes: Option<Vec<StrokeEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<UserId>>,
}
