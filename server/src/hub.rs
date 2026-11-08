//! Single task that owns `State` and applies chat rules. Sessions talk to it via `Event`s.

#[cfg(test)]
mod testkit;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use tokio::sync::{Semaphore, mpsc};

use crate::ai;
use crate::state::{AiJob, ConnId, Outbox, State};
use protocol::{ClientMsg, ErrorKind, RoomRef, ServerMsg};

pub enum Event {
    Connect {
        id: ConnId,
        tx: Outbox,
    },
    Msg {
        id: ConnId,
        msg: ClientMsg,
    },
    Disconnect {
        id: ConnId,
    },
    /// A bot's answer, sent back by the task that called the LLM.
    AiReply {
        group: String,
        bot: String,
        text: Result<String, String>,
    },
}

/// At most this many LLM calls in flight; extra mentions wait their turn.
const MAX_AI_CALLS: usize = 4;

/// `tx` is weak so the hub's own handle does not keep the channel open forever.
pub async fn run(tx: mpsc::WeakSender<Event>, mut rx: mpsc::Receiver<Event>) {
    let mut state = State::default();
    let limit = Arc::new(Semaphore::new(MAX_AI_CALLS));
    while let Some(ev) = rx.recv().await {
        if let Some(job) = apply(&mut state, ev) {
            spawn_ai(job, tx.clone(), limit.clone());
        }
    }
}

// The LLM takes seconds, so it runs on its own task; the hub never waits for it.
fn spawn_ai(job: AiJob, tx: mpsc::WeakSender<Event>, limit: Arc<Semaphore>) {
    tokio::spawn(async move {
        let Ok(_permit) = limit.acquire().await else {
            return; // semaphore closed: the hub is gone
        };
        let text = ai::complete(&job.system, &job.prompt).await;
        if let Some(tx) = tx.upgrade() {
            let (group, bot) = (job.group, job.bot);
            let _ = tx.send(Event::AiReply { group, bot, text }).await;
        }
    });
}

/// One hub step. Synchronous so tests can drive it without a runtime (see `testkit`).
/// Returns the LLM call to start, if a message mentioned a bot.
fn apply(state: &mut State, ev: Event) -> Option<AiJob> {
    match ev {
        Event::Connect { id, tx } => state.connect(id, tx),
        Event::Msg { id, msg } => return handle(state, id, msg),
        Event::Disconnect { id } => {
            if let Some(name) = state.disconnect(id) {
                broadcast_users(state);
                if state.remove_member(&name) {
                    broadcast_groups(state);
                }
            }
        }
        Event::AiReply { group, bot, text } => bot_reply(state, group, bot, text),
    }
    None
}

fn handle(state: &mut State, id: ConnId, msg: ClientMsg) -> Option<AiJob> {
    let Some(me) = state.name_of(id).map(str::to_owned) else {
        // Before registration, accept Register and silently ignore read receipts.
        match msg {
            ClientMsg::Register { name } => register(state, id, name),
            ClientMsg::Read { .. } => {}
            _ => reject(state, id, ErrorKind::NotRegistered),
        }
        return None;
    };

    match msg {
        ClientMsg::Register { .. } | ClientMsg::Pong => {}
        ClientMsg::Dm { to, text } => dm(state, id, me, to, text),
        ClientMsg::Read { room, upto } => relay_read(state, id, me, room, upto),
        ClientMsg::ListGroups => state.send(
            id,
            ServerMsg::Groups {
                groups: state.group_list(),
            },
        ),
        ClientMsg::CreateGroup { group } => create_group(state, id, me, group),
        ClientMsg::JoinGroup { group } => join_group(state, id, me, group),
        ClientMsg::LeaveGroup { group } => leave_group(state, id, &me, &group),
        ClientMsg::DeleteGroup { group } => delete_group(state, id, &me, group),
        ClientMsg::GroupMsg { group, text } => return group_msg(state, id, me, group, text),
        ClientMsg::Typing { room } => typing(state, id, me, room),
        // bot names reach clients inside the Groups broadcast
        ClientMsg::CreateBot {
            group,
            name,
            personality,
        } => match state.create_bot(&group, &me, name, personality) {
            Ok(()) => broadcast_groups(state),
            Err(kind) => reject(state, id, kind),
        },
        ClientMsg::RemoveBot { group, name } => match state.remove_bot(&group, &me, &name) {
            Ok(true) => broadcast_groups(state),
            Ok(false) => {} // no such bot
            Err(kind) => reject(state, id, kind),
        },
    }
    None
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
fn dm(state: &mut State, from_id: ConnId, from: String, to: String, text: String) {
    let Some(to_id) = state.id_of(&to) else {
        return reject(state, from_id, ErrorKind::UnknownUser);
    };
    let msg = ServerMsg::Dm {
        id: state.alloc_msg_id(),
        from,
        to,
        text,
    };
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

fn delete_group(state: &mut State, id: ConnId, me: &str, group: String) {
    match state.delete_group(&group, me) {
        Ok(notified) => {
            // Send GroupDeleted before the new list, so the client can still look up the group's name.
            let msg = ServerMsg::GroupDeleted { group };
            for member_id in notified.iter().filter_map(|m| state.id_of(m)) {
                state.send(member_id, msg.clone());
            }
            broadcast_groups(state);
        }
        Err(kind) => reject(state, id, kind),
    }
}

// members only; the sender's copy confirms delivery
fn group_msg(
    state: &mut State,
    id: ConnId,
    me: String,
    group: String,
    text: String,
) -> Option<AiJob> {
    let members = match state.members_for(&group, &me) {
        Ok(members) => members.clone(),
        Err(kind) => {
            reject(state, id, kind);
            return None;
        }
    };
    let message_id = state.alloc_msg_id();
    let msg = ServerMsg::GroupMsg {
        id: message_id,
        group: group.clone(),
        from: me.clone(),
        text: text.clone(),
    };
    for member_id in members.iter().filter_map(|m| state.id_of(m)) {
        state.send(member_id, msg.clone());
    }
    state.record(&group, &me, &text);
    // "Pirate is typing…" while the LLM works; the reply clears it on the client
    let job = state.ask_bot(&group, &text)?;
    let room = RoomRef::Group { group };
    let msg = ServerMsg::Typing {
        room,
        from: job.bot.clone(),
    };
    for member_id in members_of(state, &job.group) {
        state.send(member_id, msg.clone());
    }
    Some(job)
}

fn members_of(state: &State, group: &str) -> Vec<ConnId> {
    let members = state.members(group).into_iter().flatten();
    members.filter_map(|m| state.id_of(m)).collect()
}

// Same delivery as a member's message, so R11 (members only) holds for bots too.
// Not passed to `ask_bot`, so bots never trigger each other.
// Errors are shown to members but kept out of history, so they never reach the next prompt.
fn bot_reply(state: &mut State, group: String, bot: String, text: Result<String, String>) {
    if !state.has_bot(&group, &bot) {
        return;
    }
    let ok = text.is_ok();
    let text = text.unwrap_or_else(|e| format!("(bot unavailable: {e})"));
    let message_id = state.alloc_msg_id();
    let msg = ServerMsg::GroupMsg {
        id: message_id,
        group: group.clone(),
        from: bot.clone(),
        text: text.clone(),
    };
    for member_id in members_of(state, &group) {
        state.send(member_id, msg.clone());
    }
    if ok {
        state.record(&group, &bot, &text);
    }
}

// Transient signal: relayed to the other participants, never echoed, never stored.
// Every failure is a silent drop so a typing burst cannot become error spam.
fn typing(state: &State, id: ConnId, me: String, room: RoomRef) {
    match room {
        RoomRef::Dm { to } => {
            let Some(to_id) = state.id_of(&to).filter(|&to_id| to_id != id) else {
                return;
            };
            let room = RoomRef::Dm { to: me.clone() };
            state.send(to_id, ServerMsg::Typing { room, from: me });
        }
        RoomRef::Group { group } => {
            let Ok(members) = state.members_for(&group, &me) else {
                return;
            };
            let msg = ServerMsg::Typing {
                room: RoomRef::Group { group },
                from: me.clone(),
            };
            for member_id in members
                .iter()
                .filter(|m| **m != me)
                .filter_map(|m| state.id_of(m))
            {
                state.send(member_id, msg.clone());
            }
        }
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

fn relay_read(state: &State, reader_id: ConnId, by: String, room: RoomRef, upto: u64) {
    match room {
        RoomRef::Dm { to: with } if with != by => {
            if let Some(peer_id) = state.id_of(&with) {
                state.send(
                    peer_id,
                    ServerMsg::Read {
                        room: RoomRef::Dm { to: by.clone() },
                        by,
                        upto,
                    },
                );
            }
        }
        RoomRef::Group { group } => {
            if let Ok(info) = state.members_for(&group, &by) {
                let msg = ServerMsg::Read {
                    room: RoomRef::Group { group },
                    by,
                    upto,
                };
                for name in info {
                    if let Some(id) = state.id_of(name).filter(|id| *id != reader_id) {
                        state.send(id, msg.clone());
                    }
                }
            }
        }
        RoomRef::Dm { .. } => {}
    }
}
