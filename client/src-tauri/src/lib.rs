use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub room_id: String,
    pub sender: String,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResult {
    pub name: String,
    pub users: Vec<String>,
    pub groups: Vec<Group>,
}

struct ChatStateInner {
    connected: bool,
    current_user: Option<String>,
    users: Vec<String>,
    groups: Vec<Group>,
    messages: HashMap<String, Vec<Message>>,
    private_rooms: HashMap<String, String>,
    next_id: u64,
}

impl ChatStateInner {
    fn new() -> Self {
        Self {
            connected: false,
            current_user: None,
            users: Vec::new(),
            groups: Vec::new(),
            messages: HashMap::new(),
            private_rooms: HashMap::new(),
            next_id: 1,
        }
    }

    fn alloc_id(&mut self, prefix: &str) -> String {
        let id = format!("{prefix}-{}", self.next_id);
        self.next_id += 1;
        id
    }

    fn require_connected(&self) -> Result<&str, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        self.current_user
            .as_deref()
            .ok_or_else(|| "Not connected".into())
    }

    fn private_key(a: &str, b: &str) -> String {
        let mut pair = [a.to_string(), b.to_string()];
        pair.sort();
        format!("{}::{}", pair[0], pair[1])
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn seed_demo_data(&mut self, me: &str) {
        self.users = vec![me.to_string(), "alice".into(), "bob".into(), "carol".into()];
        self.users.sort();
        self.users.dedup();

        let g1 = self.alloc_id("group");
        let g2 = self.alloc_id("group");
        self.groups = vec![
            Group {
                id: g1.clone(),
                name: "general".into(),
                members: vec![me.to_string(), "alice".into()],
            },
            Group {
                id: g2.clone(),
                name: "study-group".into(),
                members: vec!["bob".into(), "carol".into()],
            },
        ];

        let welcome_id = self.alloc_id("msg");
        self.messages.insert(
            g1.clone(),
            vec![Message {
                id: welcome_id,
                room_id: g1,
                sender: "alice".into(),
                text: "Welcome to Deepchat (stub mode)!".into(),
                timestamp: Self::now_ms(),
            }],
        );
        self.messages.insert(g2, Vec::new());
    }
}

pub struct ChatState(Mutex<ChatStateInner>);

fn emit_users(app: &AppHandle, users: &[String]) {
    let _ = app.emit("users-updated", users);
}

fn emit_groups(app: &AppHandle, groups: &[Group]) {
    let _ = app.emit("groups-updated", groups);
}

fn emit_message(app: &AppHandle, message: &Message) {
    let _ = app.emit("message-received", message);
}

#[tauri::command]
fn connect(
    host: String,
    port: u16,
    name: String,
    state: State<'_, ChatState>,
    app: AppHandle,
) -> Result<ConnectResult, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if host.trim().is_empty() {
        return Err("Host cannot be empty".into());
    }
    if port == 0 {
        return Err("Port must be non-zero".into());
    }

    let mut inner = state.0.lock().map_err(|e| e.to_string())?;
    if inner.connected {
        return Err("Already connected".into());
    }

    // Stub: ignore real TCP; seed local demo data for UI development.
    let _ = (host, port);
    inner.connected = true;
    inner.current_user = Some(name.clone());
    inner.seed_demo_data(&name);

    let result = ConnectResult {
        name: name.clone(),
        users: inner.users.clone(),
        groups: inner.groups.clone(),
    };

    emit_users(&app, &result.users);
    emit_groups(&app, &result.groups);
    Ok(result)
}

#[tauri::command]
fn list_users(state: State<'_, ChatState>) -> Result<Vec<String>, String> {
    let inner = state.0.lock().map_err(|e| e.to_string())?;
    inner.require_connected()?;
    Ok(inner.users.clone())
}

#[tauri::command]
fn list_groups(state: State<'_, ChatState>) -> Result<Vec<Group>, String> {
    let inner = state.0.lock().map_err(|e| e.to_string())?;
    inner.require_connected()?;
    Ok(inner.groups.clone())
}

#[tauri::command]
fn create_group(
    name: String,
    state: State<'_, ChatState>,
    app: AppHandle,
) -> Result<Group, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Group name cannot be empty".into());
    }

    let mut inner = state.0.lock().map_err(|e| e.to_string())?;
    let me = inner.require_connected()?.to_string();

    let group_id = inner.alloc_id("group");
    let group = Group {
        id: group_id.clone(),
        name,
        members: vec![me],
    };
    inner.groups.push(group.clone());
    inner.messages.insert(group_id, Vec::new());

    emit_groups(&app, &inner.groups);
    Ok(group)
}

#[tauri::command]
fn join_group(
    group_id: String,
    state: State<'_, ChatState>,
    app: AppHandle,
) -> Result<Group, String> {
    let mut inner = state.0.lock().map_err(|e| e.to_string())?;
    let me = inner.require_connected()?.to_string();

    let group = inner
        .groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| "Group not found".to_string())?;

    if !group.members.iter().any(|m| m == &me) {
        group.members.push(me);
        group.members.sort();
    }

    let group = group.clone();
    emit_groups(&app, &inner.groups);
    Ok(group)
}

#[tauri::command]
fn open_private(peer: String, state: State<'_, ChatState>) -> Result<String, String> {
    let peer = peer.trim().to_string();
    let mut inner = state.0.lock().map_err(|e| e.to_string())?;
    let me = inner.require_connected()?.to_string();

    if peer.is_empty() {
        return Err("Peer name cannot be empty".into());
    }
    if peer == me {
        return Err("Cannot open a private chat with yourself".into());
    }
    if !inner.users.iter().any(|u| u == &peer) {
        return Err("Peer is not connected".into());
    }

    let key = ChatStateInner::private_key(&me, &peer);
    if let Some(room_id) = inner.private_rooms.get(&key) {
        return Ok(room_id.clone());
    }

    let room_id = inner.alloc_id("dm");
    inner.private_rooms.insert(key, room_id.clone());
    inner.messages.entry(room_id.clone()).or_default();
    Ok(room_id)
}

#[tauri::command]
fn send_message(
    room_id: String,
    text: String,
    state: State<'_, ChatState>,
    app: AppHandle,
) -> Result<Message, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Message cannot be empty".into());
    }

    let mut inner = state.0.lock().map_err(|e| e.to_string())?;
    let me = inner.require_connected()?.to_string();

    if !inner.messages.contains_key(&room_id) {
        // Allow sending into known group rooms even if empty map entry missing.
        let is_group = inner.groups.iter().any(|g| g.id == room_id);
        let is_private = inner.private_rooms.values().any(|id| id == &room_id);
        if !is_group && !is_private {
            return Err("Room not found".into());
        }
        inner.messages.insert(room_id.clone(), Vec::new());
    }

    if let Some(group) = inner.groups.iter().find(|g| g.id == room_id)
        && !group.members.iter().any(|m| m == &me)
    {
        return Err("You are not a member of this group".into());
    }

    let message = Message {
        id: inner.alloc_id("msg"),
        room_id: room_id.clone(),
        sender: me,
        text,
        timestamp: ChatStateInner::now_ms(),
    };
    inner
        .messages
        .entry(room_id)
        .or_default()
        .push(message.clone());

    emit_message(&app, &message);
    Ok(message)
}

#[tauri::command]
fn get_messages(room_id: String, state: State<'_, ChatState>) -> Result<Vec<Message>, String> {
    let inner = state.0.lock().map_err(|e| e.to_string())?;
    inner.require_connected()?;
    Ok(inner.messages.get(&room_id).cloned().unwrap_or_default())
}

/// Demo helper: inject a fake incoming message into the active room.
#[tauri::command]
fn simulate_incoming(
    room_id: String,
    state: State<'_, ChatState>,
    app: AppHandle,
) -> Result<Message, String> {
    let mut inner = state.0.lock().map_err(|e| e.to_string())?;
    let me = inner.require_connected()?.to_string();

    if !inner.messages.contains_key(&room_id)
        && !inner.groups.iter().any(|g| g.id == room_id)
        && !inner.private_rooms.values().any(|id| id == &room_id)
    {
        return Err("Room not found".into());
    }

    let sender = inner
        .users
        .iter()
        .find(|u| *u != &me)
        .cloned()
        .unwrap_or_else(|| "alice".into());

    let message = Message {
        id: inner.alloc_id("msg"),
        room_id: room_id.clone(),
        sender,
        text: "Hey — this is a simulated incoming message.".into(),
        timestamp: ChatStateInner::now_ms(),
    };
    inner
        .messages
        .entry(room_id)
        .or_default()
        .push(message.clone());

    emit_message(&app, &message);
    Ok(message)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ChatState(Mutex::new(ChatStateInner::new())))
        .invoke_handler(tauri::generate_handler![
            connect,
            list_users,
            list_groups,
            create_group,
            join_group,
            open_private,
            send_message,
            get_messages,
            simulate_incoming,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
