use super::testkit::{Harness, dm, error, got_dm};
use super::*;

fn read(room: RoomRef, upto: u64) -> ClientMsg {
    ClientMsg::Read { room, upto }
}
fn dm_room(with: &str) -> RoomRef {
    RoomRef::Dm { to: with.into() }
}
fn group_room(group: &str) -> RoomRef {
    RoomRef::Group {
        group: group.into(),
    }
}

#[test]
fn ids_increase_across_rooms_and_bots_without_burning_ids_on_rejection() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, dm("missing", "rejected"));
    assert_eq!(a.drain(), [error(ErrorKind::UnknownUser)]);
    h.send(
        &a,
        ClientMsg::GroupMsg {
            group: "missing".into(),
            text: "rejected".into(),
        },
    );
    a.drain();
    h.send(&a, ClientMsg::CreateGroup { group: "g".into() });
    a.drain();
    b.drain();
    c.drain();
    h.send(
        &c,
        ClientMsg::GroupMsg {
            group: "g".into(),
            text: "rejected".into(),
        },
    );
    assert_eq!(c.drain(), [error(ErrorKind::NotMember)]);
    h.send(&a, dm("b", "one"));
    assert_eq!(a.drain(), [got_dm("a", "b", "one", 1)]);
    assert_eq!(b.drain(), [got_dm("a", "b", "one", 1)]);
    h.send(
        &a,
        ClientMsg::GroupMsg {
            group: "g".into(),
            text: "two".into(),
        },
    );
    assert!(matches!(
        a.drain().as_slice(),
        [ServerMsg::GroupMsg { id: 2, .. }]
    ));
    h.send(
        &a,
        ClientMsg::CreateBot {
            group: "g".into(),
            name: "Pirate".into(),
            personality: "system".into(),
        },
    );
    a.drain();
    b.drain();
    c.drain();
    h.send(
        &a,
        ClientMsg::GroupMsg {
            group: "g".into(),
            text: "@Pirate hi".into(),
        },
    );
    assert!(matches!(
        a.drain().as_slice(),
        [ServerMsg::GroupMsg { id: 3, .. }, ServerMsg::Typing { .. }]
    ));
    let job = h.ai_jobs().pop().unwrap();
    h.apply(Event::AiReply {
        group: job.group,
        bot: job.bot,
        text: "Ahoy".into(),
    });
    assert!(matches!(
        a.drain().as_slice(),
        [ServerMsg::GroupMsg { id: 4, .. }]
    ));
    h.send(&a, dm("c", "five"));
    assert_eq!(a.drain(), [got_dm("a", "c", "five", 5)]);
    assert_eq!(c.drain(), [got_dm("a", "c", "five", 5)]);
    b.assert_silent();
}

#[test]
fn dm_receipt_names_the_reader_as_peer_and_never_echoes() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, dm("b", "hi"));
    a.drain();
    b.drain();
    h.send(&b, read(dm_room("a"), 1));
    assert_eq!(
        a.drain(),
        [ServerMsg::Read {
            room: dm_room("b"),
            by: "b".into(),
            upto: 1
        }]
    );
    b.assert_silent();
    c.assert_silent();
    for room in [dm_room("b"), dm_room("offline"), group_room("missing")] {
        h.send(&b, read(room, 1));
    }
    a.assert_silent();
    b.assert_silent();
    c.assert_silent();
    let mut idle = h.connect();
    h.send(&idle, read(dm_room("a"), 1));
    idle.assert_silent();
    a.assert_silent();
}

#[test]
fn group_receipts_reach_other_members_only_and_invalid_readers_are_silent() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c, mut d] = h.join(["a", "b", "c", "d"]);
    h.send(&a, ClientMsg::CreateGroup { group: "g".into() });
    for client in [&b, &c] {
        h.send(client, ClientMsg::JoinGroup { group: "g".into() });
    }
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.drain();
    }
    h.send(&b, read(group_room("g"), 7));
    let expected = ServerMsg::Read {
        room: group_room("g"),
        by: "b".into(),
        upto: 7,
    };
    assert_eq!(a.drain().as_slice(), std::slice::from_ref(&expected));
    assert_eq!(c.drain(), [expected]);
    b.assert_silent();
    d.assert_silent();
    h.send(&d, read(group_room("g"), 7));
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.assert_silent();
    }
    h.send(&b, ClientMsg::LeaveGroup { group: "g".into() });
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.drain();
    }
    h.send(&b, read(group_room("g"), 7));
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.assert_silent();
    }
}
