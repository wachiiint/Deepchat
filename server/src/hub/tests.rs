//! Hub behavior tests, driven through `testkit::Harness`.

use super::testkit::{Harness, dm, error, got_dm, users, welcome};
use super::*;

// --- register and user list ---

#[test]
fn register_sends_welcome_then_users() {
    let mut h = Harness::default();
    let mut a = h.register("a");
    assert_eq!(a.drain(), [welcome("a"), users(&["a"])]);
}

#[test]
fn users_broadcast_reaches_everyone_sorted() {
    let mut h = Harness::default();
    let mut b = h.register("b");
    let mut a = h.register("a");
    assert_eq!(a.drain(), [welcome("a"), users(&["a", "b"])]);
    assert_eq!(b.drain(), [welcome("b"), users(&["b"]), users(&["a", "b"])]);
}

#[test]
fn unregistered_conn_gets_no_broadcast() {
    let mut h = Harness::default();
    let mut idle = h.connect();
    h.register("a");
    idle.assert_silent();
}

#[test]
fn connect_alone_emits_nothing() {
    let mut h = Harness::default();
    h.connect().assert_silent();
}

// --- NameTaken ---

#[test]
fn duplicate_name_rejected() {
    let mut h = Harness::default();
    let [mut a] = h.join(["a"]);
    let mut b = h.connect();
    h.register_as(&b, "a");
    assert_eq!(b.drain(), [error(ErrorKind::NameTaken)]);
    a.assert_silent();
}

#[test]
fn rejected_conn_stays_unregistered() {
    let mut h = Harness::default();
    let [mut a] = h.join(["a"]);
    let mut b = h.connect();
    h.register_as(&b, "a");
    b.drain();
    h.send(&b, dm("a", "hi"));
    assert_eq!(b.drain(), [error(ErrorKind::NotRegistered)]);
    a.assert_silent();
}

#[test]
fn can_retry_with_another_name() {
    let mut h = Harness::default();
    h.join(["a"]);
    let mut b = h.connect();
    h.register_as(&b, "a");
    b.drain();
    h.register_as(&b, "b");
    assert_eq!(b.drain(), [welcome("b"), users(&["a", "b"])]);
}

#[test]
fn name_is_freed_on_disconnect() {
    let mut h = Harness::default();
    let [a] = h.join(["a"]);
    h.disconnect(&a);
    let mut a2 = h.register("a");
    assert_eq!(a2.drain(), [welcome("a"), users(&["a"])]);
}

// --- NotRegistered ---

#[test]
fn dm_before_register_is_rejected() {
    let mut h = Harness::default();
    let [mut a, mut b] = h.join(["a", "b"]);
    let mut idle = h.connect();
    h.send(&idle, dm("b", "hi"));
    assert_eq!(idle.drain(), [error(ErrorKind::NotRegistered)]);
    a.assert_silent();
    b.assert_silent();
}

#[test]
fn group_msg_before_register_is_rejected() {
    let mut h = Harness::default();
    let mut idle = h.connect();
    let msg = ClientMsg::GroupMsg {
        group: "g".into(),
        text: "hi".into(),
    };
    h.send(&idle, msg);
    assert_eq!(idle.drain(), [error(ErrorKind::NotRegistered)]);
}

// --- DM ---

#[test]
fn dm_reaches_only_sender_and_receiver() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, dm("b", "hi"));
    assert_eq!(a.drain(), [got_dm("a", "b", "hi")]);
    assert_eq!(b.drain(), [got_dm("a", "b", "hi")]);
    c.assert_silent();
}

#[test]
fn dm_to_self_delivers_once() {
    let mut h = Harness::default();
    let [mut a] = h.join(["a"]);
    h.send(&a, dm("a", "me"));
    assert_eq!(a.drain(), [got_dm("a", "a", "me")]);
}

#[test]
fn dm_unknown_user() {
    let mut h = Harness::default();
    let [mut a, mut b] = h.join(["a", "b"]);
    h.send(&a, dm("zed", "hi"));
    assert_eq!(a.drain(), [error(ErrorKind::UnknownUser)]);
    b.assert_silent();
}

#[test]
fn dm_to_disconnected_user() {
    let mut h = Harness::default();
    let [mut a, b] = h.join(["a", "b"]);
    h.disconnect(&b);
    a.drain();
    h.send(&a, dm("b", "hi"));
    assert_eq!(a.drain(), [error(ErrorKind::UnknownUser)]);
}

#[test]
fn dm_order_preserved() {
    let mut h = Harness::default();
    let [a, mut b] = h.join(["a", "b"]);
    h.send(&a, dm("b", "1"));
    h.send(&a, dm("b", "2"));
    assert_eq!(b.drain(), [got_dm("a", "b", "1"), got_dm("a", "b", "2")]);
}

// --- Disconnect ---

#[test]
fn disconnect_broadcasts_users_without_leaver() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.disconnect(&c);
    assert_eq!(a.drain(), [users(&["a", "b"])]);
    assert_eq!(b.drain(), [users(&["a", "b"])]);
    c.assert_silent();
}

#[test]
fn unregistered_disconnect_is_silent() {
    let mut h = Harness::default();
    let [mut a] = h.join(["a"]);
    let idle = h.connect();
    h.disconnect(&idle);
    a.assert_silent();
}

#[test]
fn unknown_or_repeated_disconnect_is_noop() {
    let mut h = Harness::default();
    let [a, mut b] = h.join(["a", "b"]);
    h.apply(Event::Disconnect { id: 999 });
    h.disconnect(&a);
    h.disconnect(&a);
    assert_eq!(b.drain(), [users(&["b"])]);
}

#[test]
fn dead_outbox_does_not_block_others() {
    let mut h = Harness::default();
    let [mut a, b, mut c] = h.join(["a", "b", "c"]);
    drop(b); // writer task gone, Disconnect not processed yet
    h.send(&a, dm("b", "hi"));
    assert_eq!(a.drain(), [got_dm("a", "b", "hi")]);
    h.register("d");
    // don't pin whether the dead `b` is still listed, only that live clients are served
    let lists_d =
        |m: &ServerMsg| matches!(m, ServerMsg::Users { names } if names.contains(&"d".to_string()));
    assert!(a.drain().iter().any(lists_d));
    assert!(c.drain().iter().any(lists_d));
}

// --- plumbing ---

#[tokio::test]
async fn run_applies_events_in_order() {
    let (ev_tx, ev_rx) = mpsc::channel(8);
    let (tx, mut rx) = mpsc::unbounded_channel();
    ev_tx.send(Event::Connect { id: 0, tx }).await.unwrap();
    let msg = ClientMsg::Register { name: "a".into() };
    ev_tx.send(Event::Msg { id: 0, msg }).await.unwrap();
    drop(ev_tx); // closes the channel, so run() returns
    run(ev_rx).await;
    assert_eq!(rx.recv().await, Some(welcome("a")));
    assert_eq!(rx.recv().await, Some(users(&["a"])));
}
