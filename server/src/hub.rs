//! Single task that owns `State` and applies chat rules. Sessions talk to it via `Event`s.

#[cfg(test)]
mod testkit;
#[cfg(test)]
mod tests;

use tokio::sync::mpsc;

use crate::state::{ConnId, Outbox, State};
use protocol::{ClientMsg, ErrorKind, ServerMsg};

pub enum Event {
    Connect { id: ConnId, tx: Outbox },
    Msg { id: ConnId, msg: ClientMsg },
    Disconnect { id: ConnId },
}

pub async fn run(mut rx: mpsc::Receiver<Event>) {
    let mut state = State::default();
    while let Some(ev) = rx.recv().await {
        apply(&mut state, ev);
    }
}

/// One hub step. Synchronous so tests can drive it without a runtime (see `testkit`).
fn apply(state: &mut State, ev: Event) {
    match ev {
        Event::Connect { id, tx } => state.connect(id, tx),
        Event::Msg { id, msg } => handle(state, id, msg),
        Event::Disconnect { id } => {
            if let Some(name) = state.disconnect(id) {
                broadcast_users(state);
                if state.remove_member(&name) {
                    broadcast_groups(state);
                }
            }
        }
    }
}

fn handle(state: &mut State, id: ConnId, msg: ClientMsg) {
    let Some(me) = state.name_of(id).map(str::to_owned) else {
        // before a name is set, only Register is allowed
        match msg {
            ClientMsg::Register { name } => register(state, id, name),
            _ => reject(state, id, ErrorKind::NotRegistered),
        }
        return;
    };

    match msg {
        ClientMsg::Register { .. } | ClientMsg::Pong => {}
        ClientMsg::Dm { to, text } => dm(state, id, me, to, text),
        ClientMsg::ListGroups => state.send(
            id,
            ServerMsg::Groups {
                groups: state.group_list(),
            },
        ),
        ClientMsg::CreateGroup { group } => create_group(state, id, me, group),
        ClientMsg::JoinGroup { group } => join_group(state, id, me, group),
        ClientMsg::LeaveGroup { group } => leave_group(state, id, &me, &group),
        ClientMsg::GroupMsg { group, text } => group_msg(state, id, me, group, text),
    }
}

fn register(state: &mut State, id: ConnId, name: String) {
    match state.register(id, name.clone()) {
        Ok(()) => {
            state.send(id, ServerMsg::Welcome { name });
            broadcast_users(state);
        }
        Err(kind) => reject(state, id, kind),
    }
}

// only sender and receiver get it; the sender's copy confirms delivery
fn dm(state: &State, from_id: ConnId, from: String, to: String, text: String) {
    let Some(to_id) = state.id_of(&to) else {
        return reject(state, from_id, ErrorKind::UnknownUser);
    };
    let msg = ServerMsg::Dm { from, to, text };
    if to_id != from_id {
        state.send(to_id, msg.clone());
    }
    state.send(from_id, msg);
}

fn create_group(state: &mut State, id: ConnId, me: String, group: String) {
    match state.create_group(group, &me) {
        Ok(()) => broadcast_groups(state),
        Err(kind) => reject(state, id, kind),
    }
}

fn join_group(state: &mut State, id: ConnId, me: String, group: String) {
    match state.join_group(&group, me) {
        Ok(true) => broadcast_groups(state),
        Ok(false) => {} // already a member
        Err(kind) => reject(state, id, kind),
    }
}

fn leave_group(state: &mut State, id: ConnId, me: &str, group: &str) {
    match state.leave_group(group, me) {
        Ok(()) => broadcast_groups(state),
        Err(kind) => reject(state, id, kind),
    }
}

// members only; the sender's copy confirms delivery
fn group_msg(state: &State, id: ConnId, me: String, group: String, text: String) {
    let members = match state.members_for(&group, &me) {
        Ok(members) => members,
        Err(kind) => return reject(state, id, kind),
    };
    let msg = ServerMsg::GroupMsg {
        group,
        from: me,
        text,
    };
    for member_id in members.iter().filter_map(|m| state.id_of(m)) {
        state.send(member_id, msg.clone());
    }
}

fn reject(state: &State, id: ConnId, kind: ErrorKind) {
    state.send(id, ServerMsg::Error { kind });
}

fn broadcast_users(state: &State) {
    state.broadcast(&ServerMsg::Users {
        names: state.names(),
    });
}

fn broadcast_groups(state: &State) {
    state.broadcast(&ServerMsg::Groups {
        groups: state.group_list(),
    });
}
