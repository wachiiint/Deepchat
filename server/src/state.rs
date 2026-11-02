//! In-memory server state. Owned by the hub task only, so no locks.

use std::collections::{BTreeSet, HashMap};

use tokio::sync::mpsc;

use protocol::{ErrorKind, GroupInfo, ServerMsg};

pub type ConnId = u64;
pub type Outbox = mpsc::UnboundedSender<ServerMsg>;

struct Conn {
    tx: Outbox,
    name: Option<String>, // None until Register succeeds
}

pub struct Group {
    pub creator: String,
    pub members: BTreeSet<String>,
}

#[derive(Default)]
pub struct State {
    conns: HashMap<ConnId, Conn>,
    names: HashMap<String, ConnId>,
    pub groups: HashMap<String, Group>,
}

impl State {
    pub fn connect(&mut self, id: ConnId, tx: Outbox) {
        self.conns.insert(id, Conn { tx, name: None });
    }

    /// Returns the name if the connection had registered.
    pub fn disconnect(&mut self, id: ConnId) -> Option<String> {
        let name = self.conns.remove(&id)?.name?;
        self.names.remove(&name);
        Some(name)
    }

    pub fn register(&mut self, id: ConnId, name: String) -> Result<(), ErrorKind> {
        if self.names.contains_key(&name) {
            return Err(ErrorKind::NameTaken);
        }
        let Some(conn) = self.conns.get_mut(&id) else {
            return Ok(()); // already disconnected
        };
        conn.name = Some(name.clone());
        self.names.insert(name, id);
        Ok(())
    }

    pub fn name_of(&self, id: ConnId) -> Option<&str> {
        self.conns.get(&id)?.name.as_deref()
    }

    pub fn id_of(&self, name: &str) -> Option<ConnId> {
        self.names.get(name).copied()
    }

    /// Registered names, sorted so every client shows the same order.
    pub fn names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.names.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn group_list(&self) -> Vec<GroupInfo> {
        let mut groups: Vec<_> = self
            .groups
            .iter()
            .map(|(name, group)| GroupInfo {
                group: name.clone(),
                creator: group.creator.clone(),
                members: group.members.iter().cloned().collect(),
            })
            .collect();
        groups.sort_by(|a, b| a.group.cmp(&b.group));
        groups
    }

    /// Remove memberships, dropping all state owned by an emptied group.
    pub fn remove_member(&mut self, name: &str) -> bool {
        let mut changed = false;
        self.groups.retain(|_, group| {
            changed |= group.members.remove(name);
            !group.members.is_empty()
        });
        changed
    }

    pub fn send(&self, id: ConnId, msg: ServerMsg) {
        if let Some(conn) = self.conns.get(&id) {
            // Err = writer task already gone; Disconnect event will clean up
            let _ = conn.tx.send(msg);
        }
    }

    pub fn broadcast(&self, msg: &ServerMsg) {
        for &id in self.names.values() {
            self.send(id, msg.clone());
        }
    }
}
