mod ai;
mod hub;
mod session;
mod state;

#[cfg(test)]
mod group_tcp_tests;

use tokio::net::TcpListener;
use tokio::sync::mpsc;

use state::ConnId;

// 0.0.0.0 = accept connections from other machines, not just localhost
const ADDR: &str = "0.0.0.0:7878";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDR).await?;
    println!("Deepchat! by deepaung, listening on {ADDR}");

    // bounded: if the hub falls behind, readers wait instead of piling up memory
    let (hub_tx, hub_rx) = mpsc::channel(1024);
    tokio::spawn(hub::run(hub_tx.downgrade(), hub_rx));

    let mut next_id: ConnId = 0;
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                println!("+ {peer} connected");
                tokio::spawn(session::run(
                    stream,
                    peer,
                    next_id,
                    hub_tx.clone(),
                    session::Liveness::DEFAULT,
                ));
                next_id += 1;
            }
            // e.g. out of file descriptors; skip this client, keep serving others
            Err(e) => eprintln!("accept failed: {e}"),
        }
    }
}
