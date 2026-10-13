mod net;
mod protocol;

use std::net::SocketAddr;

use net::framer::read_frame;
use protocol::ClientMsg;
use tokio::net::{TcpListener, TcpStream};

// 0.0.0.0 = accept connections from other machines, not just localhost
const ADDR: &str = "0.0.0.0:7878";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDR).await?;
    println!("Deepchat! by deepaung, listening on {ADDR}");

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                println!("+ {peer} connected");
                tokio::spawn(handle(stream, peer));
            }
            // e.g. out of file descriptors; skip this client, keep serving others
            Err(e) => eprintln!("accept failed: {e}"),
        }
    }
}

// one task per client: read frames until it leaves
async fn handle(mut stream: TcpStream, peer: SocketAddr) {
    // send small frames immediately instead of letting Nagle batch them
    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("{peer} set_nodelay failed: {e}");
    }

    loop {
        match read_frame::<_, ClientMsg>(&mut stream).await {
            Ok(Some(msg)) => println!("{peer} -> {msg:?}"),
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
}
