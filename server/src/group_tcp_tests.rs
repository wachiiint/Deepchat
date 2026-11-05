//! Real TCP coverage of the group lifecycle with three independent clients.
use crate::{hub, session};
use protocol::{ClientMsg, ErrorKind, GroupInfo, ServerMsg, read_frame, write_frame};
use std::time::Duration;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::timeout,
};

const LIMIT: Duration = Duration::from_secs(5);

async fn receive(socket: &mut TcpStream) -> ServerMsg {
    timeout(LIMIT, read_frame(socket))
        .await
        .unwrap()
        .unwrap()
        .unwrap()
}

async fn send(socket: &mut TcpStream, msg: ClientMsg) {
    write_frame(socket, &msg).await.unwrap();
}

async fn list(socket: &mut TcpStream) -> Vec<GroupInfo> {
    send(socket, ClientMsg::ListGroups).await;
    next_groups(socket).await
}

async fn next_groups(socket: &mut TcpStream) -> Vec<GroupInfo> {
    loop {
        if let ServerMsg::Groups { groups } = receive(socket).await {
            return groups;
        }
    }
}

async fn register(addr: std::net::SocketAddr, name: &str) -> TcpStream {
    let mut socket = TcpStream::connect(addr).await.unwrap();
    socket.set_nodelay(true).unwrap();
    send(&mut socket, ClientMsg::Register { name: name.into() }).await;
    assert_eq!(
        receive(&mut socket).await,
        ServerMsg::Welcome { name: name.into() }
    );
    assert!(matches!(
        receive(&mut socket).await,
        ServerMsg::Users { .. }
    ));
    socket
}

#[tokio::test]
async fn three_clients_create_join_chat_leave_disconnect_and_recreate_over_tcp() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (hub_tx, hub_rx) = mpsc::channel(32);
    let hub_task = tokio::spawn(hub::run(hub_tx.downgrade(), hub_rx));
    let accept_task = tokio::spawn(async move {
        let mut id = 0;
        while let Ok((socket, peer)) = listener.accept().await {
            tokio::spawn(session::run(
                socket,
                peer,
                id,
                hub_tx.clone(),
                session::Liveness::DEFAULT,
            ));
            id += 1;
        }
    });

    // A, B and C register
    let mut a = register(addr, "A").await;
    let mut b = register(addr, "B").await;
    let mut c = register(addr, "C").await;

    // ListGroups acts as a barrier for preceding presence updates on each connection.
    assert!(list(&mut a).await.is_empty());
    assert!(list(&mut b).await.is_empty());
    assert!(list(&mut c).await.is_empty());

    send(&mut a, ClientMsg::CreateGroup { group: "g1".into() }).await;
    let created = vec![GroupInfo {
        group: "g1".into(),
        creator: "A".into(),
        members: vec!["A".into()],
        bots: vec![],
    }];
    for socket in [&mut a, &mut b, &mut c] {
        assert_eq!(next_groups(socket).await, created);
    }

    send(&mut b, ClientMsg::JoinGroup { group: "g1".into() }).await;
    let joined = vec![GroupInfo {
        members: vec!["A".into(), "B".into()],
        bots: vec![],
        ..created[0].clone()
    }];
    for socket in [&mut a, &mut b, &mut c] {
        assert_eq!(next_groups(socket).await, joined);
    }

    send(
        &mut a,
        ClientMsg::GroupMsg {
            group: "g1".into(),
            text: "hello 🌏".into(),
        },
    )
    .await;
    let message = ServerMsg::GroupMsg {
        group: "g1".into(),
        from: "A".into(),
        text: "hello 🌏".into(),
    };
    assert_eq!(receive(&mut a).await, message);
    assert_eq!(receive(&mut b).await, message);

    // Every broadcast would precede C's response. No timing-based silence assertion needed.
    send(&mut c, ClientMsg::ListGroups).await;
    assert_eq!(receive(&mut c).await, ServerMsg::Groups { groups: joined });
    send(
        &mut c,
        ClientMsg::GroupMsg {
            group: "g1".into(),
            text: "blocked".into(),
        },
    )
    .await;
    assert_eq!(
        receive(&mut c).await,
        ServerMsg::Error {
            kind: ErrorKind::NotMember
        }
    );

    send(&mut a, ClientMsg::LeaveGroup { group: "g1".into() }).await;
    let remaining = vec![GroupInfo {
        members: vec!["B".into()],
        bots: vec![],
        ..created[0].clone()
    }];
    for socket in [&mut a, &mut b, &mut c] {
        assert_eq!(next_groups(socket).await, remaining);
    }

    // B's disconnect deletes the now-empty group
    drop(b);
    assert!(next_groups(&mut a).await.is_empty());
    assert!(next_groups(&mut c).await.is_empty());

    send(&mut c, ClientMsg::CreateGroup { group: "g1".into() }).await;
    assert_eq!(
        next_groups(&mut c).await,
        vec![GroupInfo {
            group: "g1".into(),
            creator: "C".into(),
            members: vec!["C".into()],
            bots: vec![]
        }]
    );

    // only the creator may delete; A first skips the list push for C's re-creation
    next_groups(&mut a).await;
    send(&mut a, ClientMsg::DeleteGroup { group: "g1".into() }).await;
    assert_eq!(
        receive(&mut a).await,
        ServerMsg::Error {
            kind: ErrorKind::NotCreator
        }
    );
    send(&mut c, ClientMsg::DeleteGroup { group: "g1".into() }).await;
    assert_eq!(
        receive(&mut c).await,
        ServerMsg::GroupDeleted { group: "g1".into() }
    );
    assert!(next_groups(&mut c).await.is_empty());
    assert!(next_groups(&mut a).await.is_empty());

    drop(a);
    drop(c);
    accept_task.abort();
    hub_task.abort();
}
