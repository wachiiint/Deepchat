//! Per-connection session: the bridge between one client's socket and the hub.
//!
//! [`run`] drives a connection from accept to disconnect. A reader loop turns incoming
//! frames into hub [`Event`]s, and a writer task turns the client's outbox into outgoing
//! frames. The session decides nothing about chat rules; those live in the hub.
//!
//! The session also keeps the connection honest, see [`Liveness`]: the writer pings the
//! client periodically, and the reader drops it once nothing has arrived for too long.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{self, Instant};

use crate::hub::Event;
use crate::state::ConnId;
use protocol::{ClientMsg, ServerMsg, read_frame, write_frame};

/// Liveness timing. The `timeout` is what detects a dead client; `Ping` only gives an idle
/// client something to answer so its silence is not mistaken for death.
#[derive(Clone, Copy)]
pub struct Liveness {
    /// How often the server sends `Ping`.
    pub ping_every: Duration,
    /// Drop the client when no frame arrives for this long. Keep it above `ping_every`
    /// (we use 3x) so one lost or late `Pong` does not kick a healthy client.
    pub timeout: Duration,
}

impl Liveness {
    pub const DEFAULT: Self = Self {
        ping_every: Duration::from_secs(10),
        timeout: Duration::from_secs(30),
    };
}

pub async fn run(
    stream: TcpStream,
    peer: SocketAddr,
    id: ConnId,
    hub: mpsc::Sender<Event>,
    live: Liveness,
) {
    // send small frames immediately instead of letting Nagle batch them
    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("{peer} set_nodelay failed: {e}");
    }
    let (mut rd, mut wr) = stream.into_split();

    // unbounded: the hub must never wait on one slow client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMsg>();
    if hub.send(Event::Connect { id, tx }).await.is_err() {
        return; // hub is gone, server shutting down
    }

    // Writer ends when the hub drops this client's Outbox, or when aborted below.
    // Pings are written here, not from the reader loop: `read_frame` is not cancel-safe,
    // so racing it against a timer could cut a frame in half and corrupt the stream.
    let writer = tokio::spawn(async move {
        // first ping after one period, not at connect
        let mut ping = time::interval_at(Instant::now() + live.ping_every, live.ping_every);
        loop {
            let msg = tokio::select! {
                m = rx.recv() => match m {
                    Some(m) => m,
                    None => break,
                },
                _ = ping.tick() => ServerMsg::Ping,
            };
            if let Err(e) = write_frame(&mut wr, &msg).await {
                eprintln!("! {peer} write failed: {e}");
                break;
            }
        }
    });

    loop {
        // the timeout restarts on every frame, so any traffic counts as alive, not just Pong
        match time::timeout(live.timeout, read_frame::<_, ClientMsg>(&mut rd)).await {
            Ok(Ok(Some(ClientMsg::Pong))) => {} // liveness only, the hub has no use for it
            Ok(Ok(Some(msg))) => {
                println!("{peer} -> {msg:?}");
                if hub.send(Event::Msg { id, msg }).await.is_err() {
                    break;
                }
            }
            Ok(Ok(None)) => {
                println!("- {peer} disconnected");
                break;
            }
            Ok(Err(e)) => {
                eprintln!("! {peer} dropped: {e}");
                break;
            }
            Err(_) => {
                eprintln!("! {peer} timed out: no frame for {:?}", live.timeout);
                break;
            }
        }
    }
    let _ = hub.send(Event::Disconnect { id }).await;
    // A dead peer can leave the writer stuck in write_all for minutes; abort it so the
    // socket really closes now.
    writer.abort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    const FAST: Liveness = Liveness {
        ping_every: Duration::from_millis(50),
        timeout: Duration::from_millis(200),
    };
    // generous bound so a slow CI box does not flake; only used to fail instead of hang
    const LIMIT: Duration = Duration::from_secs(5);

    /// Starts one session against a raw client socket; returns the client end, the hub's inbox,
    /// and the Outbox the hub would hold (dropping it would stop the writer, like a hub drop).
    async fn start(live: Liveness) -> (TcpStream, mpsc::Receiver<Event>, crate::state::Outbox) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).await.unwrap();
        let (server_side, peer) = listener.accept().await.unwrap();
        let (hub_tx, mut hub_rx) = mpsc::channel(16);
        tokio::spawn(run(server_side, peer, 0, hub_tx, live));
        let Some(Event::Connect { id: 0, tx }) = hub_rx.recv().await else {
            panic!("expected Connect");
        };
        (client, hub_rx, tx)
    }

    #[tokio::test]
    async fn silent_client_gets_pinged_then_dropped() {
        let (mut client, mut hub, _outbox) = start(FAST).await;
        let ping = time::timeout(LIMIT, read_frame::<_, ServerMsg>(&mut client)).await;
        assert!(matches!(ping, Ok(Ok(Some(ServerMsg::Ping)))));
        // never answer
        let ev = time::timeout(LIMIT, hub.recv()).await.unwrap();
        assert!(matches!(ev, Some(Event::Disconnect { id: 0 })));
    }

    #[tokio::test]
    async fn client_answering_pings_stays_connected() {
        let (mut client, mut hub, _outbox) = start(FAST).await;
        // well past `timeout`: a client that only ever answers Pings must survive
        let end = Instant::now() + FAST.timeout * 4;
        while Instant::now() < end {
            let Ok(Ok(Some(msg))) =
                time::timeout(LIMIT, read_frame::<_, ServerMsg>(&mut client)).await
            else {
                panic!("server stopped pinging or closed the connection");
            };
            assert_eq!(msg, ServerMsg::Ping);
            write_frame(&mut client, &ClientMsg::Pong).await.unwrap();
        }
        // Pong never reaches the hub, and no Disconnect was sent
        assert!(hub.try_recv().is_err());
        drop(client);
        let ev = time::timeout(LIMIT, hub.recv()).await.unwrap();
        assert!(matches!(ev, Some(Event::Disconnect { id: 0 })));
    }
}
