mod connection;

use connection::{Connection, Snapshot};
use protocol::ClientMsg;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
pub struct ChatState(Mutex<Option<Arc<Connection>>>);

fn connection(state: &ChatState) -> Result<Arc<Connection>, String> {
    state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "Not connected".into())
}

#[tauri::command]
async fn connect(
    host: String,
    port: u16,
    name: String,
    state: State<'_, ChatState>,
    app: AppHandle,
) -> Result<Snapshot, String> {
    let name = name.trim().to_owned();
    if name.is_empty() || host.trim().is_empty() || port == 0 {
        return Err("Enter a display name, host, and non-zero port.".into());
    }
    // Take the previous connection out before awaiting; dropping it closes its tasks/socket.
    let previous = state.0.lock().map_err(|e| e.to_string())?.take();
    drop(previous);
    let notify = Arc::new(move |update| match update {
        connection::Update::Users(users) => {
            let _ = app.emit("users-updated", users);
        }
        connection::Update::Groups(groups) => {
            let _ = app.emit("groups-updated", groups);
        }
        connection::Update::Message(message) => {
            let _ = app.emit("message-received", message);
        }
        connection::Update::Error(message) => {
            let _ = app.emit("chat-error", message);
        }
        connection::Update::Disconnected(message) => {
            let _ = app.emit("disconnected", message);
        }
    });
    let (connection, snapshot) = Connection::connect(host.trim(), port, name, notify).await?;
    *state.0.lock().map_err(|e| e.to_string())? = Some(Arc::new(connection));
    Ok(snapshot)
}

#[tauri::command]
fn list_users(state: State<'_, ChatState>) -> Result<Vec<String>, String> {
    connection(&state)?.users()
}

#[tauri::command]
fn list_groups(state: State<'_, ChatState>) -> Result<Vec<connection::Group>, String> {
    connection(&state)?.groups()
}

#[tauri::command]
async fn create_group(name: String, state: State<'_, ChatState>) -> Result<(), String> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err("Group name cannot be empty".into());
    }
    connection(&state)?
        .send(ClientMsg::CreateGroup { group: name })
        .await
}

#[tauri::command]
async fn join_group(group_id: String, state: State<'_, ChatState>) -> Result<(), String> {
    connection(&state)?
        .send(ClientMsg::JoinGroup {
            group: connection::group_name(&group_id)?,
        })
        .await
}

#[tauri::command]
async fn leave_group(group_id: String, state: State<'_, ChatState>) -> Result<(), String> {
    connection(&state)?
        .send(ClientMsg::LeaveGroup {
            group: connection::group_name(&group_id)?,
        })
        .await
}

#[tauri::command]
fn open_private(peer: String, state: State<'_, ChatState>) -> Result<String, String> {
    connection(&state)?.open_private(&peer)
}

#[tauri::command]
async fn send_message(
    room_id: String,
    text: String,
    state: State<'_, ChatState>,
) -> Result<(), String> {
    let connection = connection(&state)?;
    let text = text.trim().to_owned();
    if text.is_empty() {
        return Err("Message cannot be empty".into());
    }
    let msg = connection.outgoing(&room_id, text)?;
    connection.send(msg).await
}

#[tauri::command]
fn get_messages(
    room_id: String,
    state: State<'_, ChatState>,
) -> Result<Vec<connection::Message>, String> {
    connection(&state)?.messages(&room_id)
}

#[tauri::command]
fn disconnect(state: State<'_, ChatState>) -> Result<(), String> {
    state.0.lock().map_err(|e| e.to_string())?.take();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ChatState::default())
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            list_users,
            list_groups,
            create_group,
            join_group,
            leave_group,
            open_private,
            send_message,
            get_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
