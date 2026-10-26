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
            if state.disconnect(id).is_some() {
                broadcast_users(state);
            }
        }
    }
}

fn handle(state: &mut State, id: ConnId, msg: ClientMsg) {
    let Some(me) = state.name_of(id).map(str::to_owned) else {
        // before a name is set, only Register is allowed
        match msg {
            ClientMsg::Register { name } => register(state, id, name),
            _ => state.send(
                id,
                ServerMsg::Error {
                    kind: ErrorKind::NotRegistered,
                },
            ),
        }
        return;
    };

    match msg {
        ClientMsg::Register { .. } | ClientMsg::Pong => {}
        ClientMsg::Dm { to, text } => dm(state, id, me, to, text),
        other => println!("{me}: not implemented yet: {other:?}"),
    }
}

fn register(state: &mut State, id: ConnId, name: String) {
    match state.register(id, name.clone()) {
        Ok(()) => {
            state.send(id, ServerMsg::Welcome { name });
            broadcast_users(state);
        }
        Err(kind) => state.send(id, ServerMsg::Error { kind }),
    }
}

// only sender and receiver get it; the sender's copy confirms delivery
fn dm(state: &State, from_id: ConnId, from: String, to: String, text: String) {
    let Some(to_id) = state.id_of(&to) else {
        state.send(
            from_id,
            ServerMsg::Error {
                kind: ErrorKind::UnknownUser,
            },
        );
        return;
    };
    let msg = ServerMsg::Dm { from, to, text };
    if to_id != from_id {
        state.send(to_id, msg.clone());
    }
    state.send(from_id, msg);
}

fn broadcast_users(state: &State) {
    state.broadcast(&ServerMsg::Users {
        names: state.names(),
    });
}
