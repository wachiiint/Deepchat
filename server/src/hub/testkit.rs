//! In-process test helper for the hub: no sockets, no runtime, no sleeps.
//!
//! `Harness` owns a `State` and feeds `Event`s straight into `hub::apply`, so when a
//! call returns the hub has fully handled it. Each `Client` is a fake connection whose
//! `Outbox` receiver you drain to see every `ServerMsg` it was sent.
//!
//! ```text
//! let mut h = Harness::default();
//! let [mut a, mut b, mut c] = h.join(["a", "b", "c"]); // registered, queues cleared
//! h.send(&a, dm("b", "hi"));
//! assert_eq!(b.drain(), [got_dm("a", "b", "hi")]);
//! c.assert_silent();
//! ```
//!
//! See `hub::tests::dm_reaches_only_sender_and_receiver` for the real version.
//! Use `register` instead of `join` to assert on the `Welcome`/`Users` replies.

use super::{Event, apply};
use crate::state::{ConnId, State};
use protocol::{ClientMsg, ErrorKind, ServerMsg};
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[derive(Default)]
pub struct Harness {
    state: State,
    next_id: ConnId,
}

/// A fake connection. Dropping it closes its outbox, like a dead writer task.
pub struct Client {
    id: ConnId,
    rx: UnboundedReceiver<ServerMsg>,
}

impl Harness {
    /// Feed any event, e.g. `Event::Disconnect` for an id that never connected.
    pub fn apply(&mut self, ev: Event) {
        apply(&mut self.state, ev);
    }

    /// `Event::Connect` only: the client has no name yet.
    pub fn connect(&mut self) -> Client {
        let id = self.next_id;
        self.next_id += 1;
        let (tx, rx) = mpsc::unbounded_channel();
        self.apply(Event::Connect { id, tx });
        Client { id, rx }
    }

    /// Connect, then send `Register`. Replies stay queued for the test to read.
    pub fn register(&mut self, name: &str) -> Client {
        let c = self.connect();
        self.register_as(&c, name);
        c
    }

    /// Send `Register` for an already connected client.
    pub fn register_as(&mut self, c: &Client, name: &str) {
        self.send(c, ClientMsg::Register { name: name.into() });
    }

    /// Register one client per name, then clear every queue so tests start from silence.
    pub fn join<const N: usize>(&mut self, names: [&str; N]) -> [Client; N] {
        let mut clients = names.map(|n| self.register(n));
        for c in &mut clients {
            c.drain();
        }
        clients
    }

    pub fn send(&mut self, c: &Client, msg: ClientMsg) {
        self.apply(Event::Msg { id: c.id, msg });
    }

    pub fn disconnect(&mut self, c: &Client) {
        self.apply(Event::Disconnect { id: c.id });
    }
}

impl Client {
    /// Everything received since the last drain, oldest first.
    pub fn drain(&mut self) -> Vec<ServerMsg> {
        let mut out = Vec::new();
        while let Ok(m) = self.rx.try_recv() {
            out.push(m);
        }
        out
    }

    pub fn assert_silent(&mut self) {
        assert_eq!(self.drain(), Vec::<ServerMsg>::new());
    }
}

// Short constructors so test bodies stay readable.

pub fn dm(to: &str, text: &str) -> ClientMsg {
    ClientMsg::Dm {
        to: to.into(),
        text: text.into(),
    }
}

pub fn welcome(name: &str) -> ServerMsg {
    ServerMsg::Welcome { name: name.into() }
}

pub fn users(names: &[&str]) -> ServerMsg {
    ServerMsg::Users {
        names: names.iter().map(|n| n.to_string()).collect(),
    }
}

pub fn got_dm(from: &str, to: &str, text: &str) -> ServerMsg {
    ServerMsg::Dm {
        from: from.into(),
        to: to.into(),
        text: text.into(),
    }
}

pub fn error(kind: ErrorKind) -> ServerMsg {
    ServerMsg::Error { kind }
}
