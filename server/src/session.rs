//! One per client: a reader loop (socket → hub) and a writer task (outbox → socket).

use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::hub::Event;
use crate::net::framer::{read_frame, write_frame};
use crate::protocol::{ClientMsg, ServerMsg};
use crate::state::ConnId;

pub async fn run(stream: TcpStream, peer: SocketAddr, id: ConnId, hub: mpsc::Sender<Event>) {
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

    // writer ends when the hub drops this client's Outbox
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = write_frame(&mut wr, &msg).await {
                eprintln!("! {peer} write failed: {e}");
                break;
            }
        }
    });

    loop {
        match read_frame::<_, ClientMsg>(&mut rd).await {
            Ok(Some(msg)) => {
                println!("{peer} -> {msg:?}");
                if hub.send(Event::Msg { id, msg }).await.is_err() {
                    break;
                }
            }
            Ok(None) => {
                println!("- {peer} disconnected");
                break;
            }
            Err(e) => {
                eprintln!("! {peer} dropped: {e}");
                break;
            }
        }
    }
    let _ = hub.send(Event::Disconnect { id }).await;
}
