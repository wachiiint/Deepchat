//! TCP connection and a cache of server-confirmed data for the desktop UI.
use protocol::{ClientMsg, ErrorKind, GroupInfo, ServerMsg, read_frame, write_frame};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, tcp::OwnedWriteHalf},
    sync::Mutex as AsyncMutex,
    task::JoinHandle,
    time::timeout,
};

const LIMIT: Duration = Duration::from_secs(10);

pub enum Update {
    Users(Vec<String>),
    Groups(Vec<Group>),
    GroupDeleted(String),
    Message(Message),
    Error(String),
    Disconnected(String),
}

type Notify = Arc<dyn Fn(Update) + Send + Sync>;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub members: Vec<String>,
}

impl From<GroupInfo> for Group {
    fn from(info: GroupInfo) -> Self {
        Self {
            id: format!("group:{}", info.group),
            name: info.group,
            creator: info.creator,
            members: info.members,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub room_id: String,
    pub sender: String,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub name: String,
    pub users: Vec<String>,
    pub groups: Vec<Group>,
}

struct Model {
    snapshot: Snapshot,
    messages: HashMap<String, Vec<Message>>,
    peers: HashMap<String, String>,
    next_id: u64,
}

pub struct Connection {
    model: Arc<Mutex<Model>>,
    writer: Arc<AsyncMutex<OwnedWriteHalf>>,
    alive: Arc<AtomicBool>,
    reader: JoinHandle<()>,
    notify: Notify,
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.reader.abort();
    }
}

pub fn group_name(id: &str) -> Result<String, String> {
    id.strip_prefix("group:")
        .map(str::to_owned)
        .ok_or_else(|| "Unknown group room".into())
}

fn private_id(a: &str, b: &str) -> String {
    let mut pair = [a, b];
    pair.sort();
    // Length prefix avoids collisions when display names contain delimiters.
    format!("dm:{}:{}{}", pair[0].len(), pair[0], pair[1])
}

fn error_text(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NameTaken => "That display name is already connected. Choose another name.",
        ErrorKind::NotRegistered => "Not registered. Reconnect to the server.",
        ErrorKind::UnknownUser => "That user is no longer connected.",
        ErrorKind::GroupExists => "A group with that name already exists. Choose another name.",
        ErrorKind::UnknownGroup => "That group no longer exists. Choose another group.",
        ErrorKind::NotMember => "You are not a member of this group.",
        ErrorKind::NotCreator => "Only the group's creator can delete it.",
    }
}

impl Model {
    fn apply(&mut self, msg: ServerMsg, notify: &Notify) {
        match msg {
            ServerMsg::Users { names } => {
                self.snapshot.users = names;
                notify(Update::Users(self.snapshot.users.clone()));
            }
            ServerMsg::Groups { groups } => {
                self.snapshot.groups = groups.into_iter().map(Group::from).collect();
                // Do not retain history across leave/delete/recreate cycles.
                let me = &self.snapshot.name;
                self.messages.retain(|room, _| {
                    !room.starts_with("group:")
                        || self
                            .snapshot
                            .groups
                            .iter()
                            .any(|g| &g.id == room && g.members.contains(me))
                });
                notify(Update::Groups(self.snapshot.groups.clone()));
            }
            ServerMsg::Dm { from, to, text } => {
                let room = private_id(&from, &to);
                let peer = if from == self.snapshot.name {
                    to
                } else {
                    from.clone()
                };
                self.peers.insert(room.clone(), peer);
                self.receive(room, from, text, notify);
            }
            ServerMsg::GroupMsg { group, from, text } => {
                self.receive(format!("group:{group}"), from, text, notify);
            }
            ServerMsg::GroupDeleted { group } => {
                let room = format!("group:{group}");
                self.messages.remove(&room);
                notify(Update::GroupDeleted(room));
            }
            ServerMsg::Error { kind } => {
                notify(Update::Error(error_text(kind).into()));
            }
            ServerMsg::Welcome { .. } | ServerMsg::Ping => {}
        }
    }

    fn receive(&mut self, room_id: String, sender: String, text: String, notify: &Notify) {
        self.next_id += 1;
        let message = Message {
            id: self.next_id.to_string(),
            room_id: room_id.clone(),
            sender,
            text,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };
        self.messages
            .entry(room_id)
            .or_default()
            .push(message.clone());
        notify(Update::Message(message));
    }
}

impl Connection {
    /// Connects and registers `name`. Returns once the server has sent `Welcome` and the
    /// first group list; anything else received meanwhile is replayed through `Model::apply`.
    pub async fn connect(
        host: &str,
        port: u16,
        name: String,
        notify: Notify,
    ) -> Result<(Self, Snapshot), String> {
        let mut socket = timeout(LIMIT, TcpStream::connect((host, port)))
            .await
            .map_err(|_| "Connection timed out".to_string())?
            .map_err(|e| e.to_string())?;
        socket.set_nodelay(true).map_err(|e| e.to_string())?;

        // registration handshake: Register, then wait for Welcome and the first group list
        let mut buffered = Vec::new();
        let snapshot = timeout(LIMIT, async {
            write_frame(&mut socket, &ClientMsg::Register { name: name.clone() })
                .await
                .map_err(|e| e.to_string())?;
            let mut snapshot = Snapshot {
                name,
                users: vec![],
                groups: vec![],
            };
            let mut welcomed = false;
            loop {
                match read_frame::<_, ServerMsg>(&mut socket)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    Some(ServerMsg::Welcome { name }) => {
                        snapshot.name = name;
                        welcomed = true;
                        write_frame(&mut socket, &ClientMsg::ListGroups)
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                    Some(ServerMsg::Users { names }) => snapshot.users = names,
                    Some(ServerMsg::Groups { groups }) if welcomed => {
                        snapshot.groups = groups.into_iter().map(Group::from).collect();
                        return Ok::<_, String>(snapshot);
                    }
                    Some(ServerMsg::Ping) => write_frame(&mut socket, &ClientMsg::Pong)
                        .await
                        .map_err(|e| e.to_string())?,
                    Some(ServerMsg::Error { kind }) => return Err(error_text(kind).into()),
                    None => return Err("Server disconnected during registration".into()),
                    Some(msg) => buffered.push(msg),
                }
            }
        })
        .await
        .map_err(|_| "Registration timed out".to_string())??;

        let model = Arc::new(Mutex::new(Model {
            snapshot: snapshot.clone(),
            messages: HashMap::new(),
            peers: HashMap::new(),
            next_id: 0,
        }));

        // frames that arrived during the handshake are replayed, not lost
        for msg in buffered {
            model.lock().map_err(|e| e.to_string())?.apply(msg, &notify);
        }

        // from here on the reader task owns the read half
        let (mut rd, wr) = socket.into_split();
        let writer = Arc::new(AsyncMutex::new(wr));
        let alive = Arc::new(AtomicBool::new(true));
        let (reader_model, reader_writer, reader_alive, reader_notify) =
            (model.clone(), writer.clone(), alive.clone(), notify.clone());
        let reader = tokio::spawn(async move {
            let reason = loop {
                match read_frame::<_, ServerMsg>(&mut rd).await {
                    Ok(Some(ServerMsg::Ping)) => {
                        if let Err(e) = timeout(LIMIT, async {
                            write_frame(&mut *reader_writer.lock().await, &ClientMsg::Pong)
                                .await
                                .map_err(|e| e.to_string())
                        })
                        .await
                        .unwrap_or_else(|_| Err("Server is not responding".into()))
                        {
                            break e;
                        }
                    }
                    Ok(Some(msg)) => {
                        if let Ok(mut model) = reader_model.lock() {
                            model.apply(msg, &reader_notify);
                        }
                    }
                    Ok(None) => break "Server disconnected. Connect again to continue.".into(),
                    Err(e) => break e.to_string(),
                }
            };
            // `swap`: if `send` already tore the connection down, it reported the disconnect.
            if reader_alive.swap(false, Ordering::AcqRel) {
                reader_notify(Update::Disconnected(reason));
            }
        });

        Ok((
            Self {
                model,
                writer,
                alive,
                reader,
                notify,
            },
            snapshot,
        ))
    }

    pub async fn send(&self, msg: ClientMsg) -> Result<(), String> {
        if !self.alive.load(Ordering::Acquire) {
            return Err("Not connected".into());
        }
        let result = timeout(LIMIT, async {
            write_frame(&mut *self.writer.lock().await, &msg)
                .await
                .map_err(|e| e.to_string())
        })
        .await
        .unwrap_or_else(|_| Err("Sending timed out. Reconnect before sending again.".into()));
        // A cancelled/failed frame write may be partial; never reuse that stream.
        // Only the first to flip `alive` tears down and reports, so the UI hears it once.
        if let Err(e) = &result
            && self.alive.swap(false, Ordering::AcqRel)
        {
            self.reader.abort();
            let _ = self.writer.lock().await.shutdown().await;
            (self.notify)(Update::Disconnected(e.clone()));
        }
        result
    }

    pub fn users(&self) -> Result<Vec<String>, String> {
        Ok(self
            .model
            .lock()
            .map_err(|e| e.to_string())?
            .snapshot
            .users
            .clone())
    }

    pub fn groups(&self) -> Result<Vec<Group>, String> {
        Ok(self
            .model
            .lock()
            .map_err(|e| e.to_string())?
            .snapshot
            .groups
            .clone())
    }

    pub fn open_private(&self, peer: &str) -> Result<String, String> {
        let mut model = self.model.lock().map_err(|e| e.to_string())?;
        if peer == model.snapshot.name || !model.snapshot.users.iter().any(|n| n == peer) {
            return Err("Choose another connected user".into());
        }
        let id = private_id(&model.snapshot.name, peer);
        model.peers.insert(id.clone(), peer.to_owned());
        Ok(id)
    }

    pub fn outgoing(&self, room: &str, text: String) -> Result<ClientMsg, String> {
        let model = self.model.lock().map_err(|e| e.to_string())?;
        if let Some(peer) = model.peers.get(room) {
            return Ok(ClientMsg::Dm {
                to: peer.clone(),
                text,
            });
        }
        let group = model
            .snapshot
            .groups
            .iter()
            .find(|g| g.id == room)
            .ok_or(error_text(ErrorKind::UnknownGroup))?;
        if !group.members.contains(&model.snapshot.name) {
            return Err(error_text(ErrorKind::NotMember).into());
        }
        Ok(ClientMsg::GroupMsg {
            group: group.name.clone(),
            text,
        })
    }

    pub fn messages(&self, room: &str) -> Result<Vec<Message>, String> {
        let model = self.model.lock().map_err(|e| e.to_string())?;
        if room.starts_with("group:")
            && !model
                .snapshot
                .groups
                .iter()
                .any(|g| g.id == room && g.members.contains(&model.snapshot.name))
        {
            return Err(error_text(ErrorKind::NotMember).into());
        }
        Ok(model.messages.get(room).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{net::TcpListener, sync::mpsc};

    async fn event(rx: &mut mpsc::UnboundedReceiver<Update>) -> Update {
        timeout(LIMIT, rx.recv()).await.unwrap().unwrap()
    }

    #[tokio::test]
    async fn tcp_connection_registers_handles_pushes_heartbeats_errors_and_disconnects() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let group = GroupInfo {
            group: "g1".into(),
            creator: "me".into(),
            members: vec!["me".into()],
        };
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            assert_eq!(
                read_frame::<_, ClientMsg>(&mut socket).await.unwrap(),
                Some(ClientMsg::Register { name: "me".into() })
            );
            write_frame(&mut socket, &ServerMsg::Welcome { name: "me".into() })
                .await
                .unwrap();
            write_frame(
                &mut socket,
                &ServerMsg::Users {
                    names: vec!["me".into(), "peer".into()],
                },
            )
            .await
            .unwrap();
            // Incoming messages during registration must be cached, not lost.
            write_frame(
                &mut socket,
                &ServerMsg::Dm {
                    from: "peer".into(),
                    to: "me".into(),
                    text: "early".into(),
                },
            )
            .await
            .unwrap();
            assert_eq!(
                read_frame::<_, ClientMsg>(&mut socket).await.unwrap(),
                Some(ClientMsg::ListGroups)
            );
            write_frame(&mut socket, &ServerMsg::Groups { groups: vec![] })
                .await
                .unwrap();
            write_frame(&mut socket, &ServerMsg::Ping).await.unwrap();
            // The heartbeat reply and user command can arrive in either order.
            let first = read_frame::<_, ClientMsg>(&mut socket)
                .await
                .unwrap()
                .unwrap();
            let second = read_frame::<_, ClientMsg>(&mut socket)
                .await
                .unwrap()
                .unwrap();
            assert!([first.clone(), second.clone()].contains(&ClientMsg::Pong));
            assert!([first, second].contains(&ClientMsg::CreateGroup { group: "g1".into() }));
            write_frame(
                &mut socket,
                &ServerMsg::Groups {
                    groups: vec![group],
                },
            )
            .await
            .unwrap();
            assert_eq!(
                read_frame::<_, ClientMsg>(&mut socket).await.unwrap(),
                Some(ClientMsg::GroupMsg {
                    group: "g1".into(),
                    text: "hi".into()
                })
            );
            write_frame(
                &mut socket,
                &ServerMsg::GroupMsg {
                    group: "g1".into(),
                    from: "me".into(),
                    text: "hi".into(),
                },
            )
            .await
            .unwrap();
            assert_eq!(
                read_frame::<_, ClientMsg>(&mut socket).await.unwrap(),
                Some(ClientMsg::LeaveGroup { group: "g1".into() })
            );
            write_frame(&mut socket, &ServerMsg::Groups { groups: vec![] })
                .await
                .unwrap();
            write_frame(
                &mut socket,
                &ServerMsg::Error {
                    kind: ErrorKind::UnknownGroup,
                },
            )
            .await
            .unwrap();
        });
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (connection, snapshot) = Connection::connect(
            "127.0.0.1",
            port,
            "me".into(),
            Arc::new(move |update| {
                tx.send(update).unwrap();
            }),
        )
        .await
        .unwrap();
        assert_eq!(snapshot.users, ["me", "peer"]);
        assert!(snapshot.groups.is_empty());
        assert!(
            matches!(event(&mut rx).await, Update::Message(message) if message.text == "early")
        );
        let private = connection.open_private("peer").unwrap();
        assert_eq!(connection.messages(&private).unwrap()[0].text, "early");
        assert!(connection.outgoing("group:g1", "hi".into()).is_err());
        connection
            .send(ClientMsg::CreateGroup { group: "g1".into() })
            .await
            .unwrap();
        assert!(
            matches!(event(&mut rx).await, Update::Groups(groups) if groups[0].creator == "me")
        );
        assert!(connection.messages("group:g1").unwrap().is_empty());
        connection
            .send(connection.outgoing("group:g1", "hi".into()).unwrap())
            .await
            .unwrap();
        assert!(
            matches!(event(&mut rx).await, Update::Message(message) if message.room_id == "group:g1" && message.text == "hi")
        );
        assert_eq!(connection.messages("group:g1").unwrap().len(), 1);
        connection
            .send(ClientMsg::LeaveGroup { group: "g1".into() })
            .await
            .unwrap();
        assert!(matches!(event(&mut rx).await, Update::Groups(groups) if groups.is_empty()));
        assert!(connection.messages("group:g1").is_err());
        assert!(
            matches!(event(&mut rx).await, Update::Error(message) if message.contains("no longer exists"))
        );
        assert!(matches!(event(&mut rx).await, Update::Disconnected(_)));
        assert!(connection.send(ClientMsg::ListGroups).await.is_err());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn registration_error_is_returned_and_socket_is_closed() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            read_frame::<_, ClientMsg>(&mut socket).await.unwrap();
            write_frame(
                &mut socket,
                &ServerMsg::Error {
                    kind: ErrorKind::NameTaken,
                },
            )
            .await
            .unwrap();
            assert_eq!(read_frame::<_, ClientMsg>(&mut socket).await.unwrap(), None);
        });
        let result = Connection::connect("127.0.0.1", port, "taken".into(), Arc::new(|_| {})).await;
        assert!(matches!(result, Err(message) if message.contains("already connected")));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn private_room_sends_dm_and_shows_only_the_server_echo() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (checked_tx, checked_rx) = tokio::sync::oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            read_frame::<_, ClientMsg>(&mut socket).await.unwrap();
            write_frame(&mut socket, &ServerMsg::Welcome { name: "me".into() })
                .await
                .unwrap();
            write_frame(
                &mut socket,
                &ServerMsg::Users {
                    names: vec!["me".into(), "peer".into()],
                },
            )
            .await
            .unwrap();
            read_frame::<_, ClientMsg>(&mut socket).await.unwrap(); // ListGroups
            write_frame(&mut socket, &ServerMsg::Groups { groups: vec![] })
                .await
                .unwrap();
            assert_eq!(
                read_frame::<_, ClientMsg>(&mut socket).await.unwrap(),
                Some(ClientMsg::Dm {
                    to: "peer".into(),
                    text: "yo".into()
                })
            );
            // Wait until the client has checked that nothing was added locally.
            checked_rx.await.unwrap();
            for (from, to, text) in [("me", "peer", "yo"), ("peer", "me", "hey")] {
                write_frame(
                    &mut socket,
                    &ServerMsg::Dm {
                        from: from.into(),
                        to: to.into(),
                        text: text.into(),
                    },
                )
                .await
                .unwrap();
            }
        });
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (connection, _) = Connection::connect(
            "127.0.0.1",
            port,
            "me".into(),
            Arc::new(move |update| {
                tx.send(update).unwrap();
            }),
        )
        .await
        .unwrap();
        // Your own name is listed but cannot be opened; neither can an unknown user.
        assert!(connection.open_private("me").is_err());
        assert!(connection.open_private("ghost").is_err());
        let private = connection.open_private("peer").unwrap();
        assert!(connection.messages(&private).unwrap().is_empty());
        connection
            .send(connection.outgoing(&private, "yo".into()).unwrap())
            .await
            .unwrap();
        // The server echoes the sender's DM, so adding it locally would show it twice.
        assert!(connection.messages(&private).unwrap().is_empty());
        checked_tx.send(()).unwrap();
        for expected in [("me", "yo"), ("peer", "hey")] {
            assert!(
                matches!(event(&mut rx).await, Update::Message(m) if m.room_id == private && (m.sender.as_str(), m.text.as_str()) == expected)
            );
        }
        let texts: Vec<_> = connection
            .messages(&private)
            .unwrap()
            .into_iter()
            .map(|m| m.text)
            .collect();
        assert_eq!(texts, ["yo", "hey"]);
        server.await.unwrap();
    }

    #[test]
    fn room_ids_are_stable_and_cannot_collide_with_delimiters_or_groups() {
        assert_eq!(private_id("a", "b"), private_id("b", "a"));
        assert_ne!(private_id("a::b", "c"), private_id("a", "b::c"));
        assert_ne!(private_id("a", "b"), "group:dm:1:ab");
    }

    #[test]
    fn group_deleted_drops_history_and_notifies_with_room_id() {
        let mut model = Model {
            snapshot: Snapshot {
                name: "me".into(),
                users: vec![],
                groups: vec![],
            },
            messages: HashMap::new(),
            peers: HashMap::new(),
            next_id: 0,
        };
        let deleted = Arc::new(Mutex::new(Vec::new()));
        let log = deleted.clone();
        let notify: Notify = Arc::new(move |update| {
            if let Update::GroupDeleted(room) = update {
                log.lock().unwrap().push(room);
            }
        });
        model.receive("group:g1".into(), "peer".into(), "hi".into(), &notify);
        assert!(model.messages.contains_key("group:g1"));
        model.apply(ServerMsg::GroupDeleted { group: "g1".into() }, &notify);
        assert!(!model.messages.contains_key("group:g1"));
        assert_eq!(*deleted.lock().unwrap(), ["group:g1"]);
    }
}
