//! Single task that owns `State` and applies chat rules. Sessions talk to it via `Event`s.

use tokio::sync::mpsc;

use crate::protocol::{ClientMsg, ErrorKind, ServerMsg};
use crate::state::{ConnId, Outbox, State};

pub enum Event {
    Connect { id: ConnId, tx: Outbox },
    Msg { id: ConnId, msg: ClientMsg },
    Disconnect { id: ConnId },
}

pub async fn run(mut rx: mpsc::Receiver<Event>) {
    let mut state = State::default();
    while let Some(ev) = rx.recv().await {
        match ev {
            Event::Connect { id, tx } => state.connect(id, tx),
            Event::Msg { id, msg } => handle(&mut state, id, msg),
            Event::Disconnect { id } => {
                if state.disconnect(id).is_some() {
                    broadcast_users(&state);
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
            _ => state.send(id, ServerMsg::Error { kind: ErrorKind::NotRegistered }),
        }
        return;
    };

    match msg {
        ClientMsg::Register { .. } | ClientMsg::Pong => {}
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

fn broadcast_users(state: &State) {
    state.broadcast(&ServerMsg::Users { names: state.names() });
}
