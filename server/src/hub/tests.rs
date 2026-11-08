//! Hub behavior tests, driven through `testkit::Harness`.

use super::testkit::{
    Harness, dm, error, got_dm, got_typing_dm, got_typing_group, typing_dm, typing_group, users,
    welcome,
};
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
    assert_eq!(a.drain(), [got_dm("a", "b", "hi", 1)]);
    assert_eq!(b.drain(), [got_dm("a", "b", "hi", 1)]);
    c.assert_silent();
}

#[test]
fn dm_to_self_delivers_once() {
    let mut h = Harness::default();
    let [mut a] = h.join(["a"]);
    h.send(&a, dm("a", "me"));
    assert_eq!(a.drain(), [got_dm("a", "a", "me", 1)]);
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
    assert_eq!(
        b.drain(),
        [got_dm("a", "b", "1", 1), got_dm("a", "b", "2", 2)]
    );
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
    assert_eq!(a.drain(), [got_dm("a", "b", "hi", 1)]);
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
    let weak = ev_tx.downgrade();
    drop(ev_tx); // closes the channel, so run() returns
    run(weak, ev_rx).await;
    assert_eq!(rx.recv().await, Some(welcome("a")));
    assert_eq!(rx.recv().await, Some(users(&["a"])));
}

// --- Groups ---

fn groups(entries: &[(&str, &str, &[&str])]) -> ServerMsg {
    ServerMsg::Groups {
        groups: entries
            .iter()
            .map(|(name, creator, members)| protocol::GroupInfo {
                group: (*name).into(),
                creator: (*creator).into(),
                members: members.iter().map(|s| s.to_string()).collect(),
                bots: vec![],
            })
            .collect(),
    }
}

fn create(group: &str) -> ClientMsg {
    ClientMsg::CreateGroup {
        group: group.into(),
    }
}
fn join_group(group: &str) -> ClientMsg {
    ClientMsg::JoinGroup {
        group: group.into(),
    }
}
fn leave(group: &str) -> ClientMsg {
    ClientMsg::LeaveGroup {
        group: group.into(),
    }
}
fn delete(group: &str) -> ClientMsg {
    ClientMsg::DeleteGroup {
        group: group.into(),
    }
}
fn group_deleted(group: &str) -> ServerMsg {
    ServerMsg::GroupDeleted {
        group: group.into(),
    }
}
fn say(group: &str, text: &str) -> ClientMsg {
    ClientMsg::GroupMsg {
        group: group.into(),
        text: text.into(),
    }
}

#[test]
fn create_broadcasts_creator_only_and_duplicate_is_rejected() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    let mut idle = h.connect();
    h.send(&a, create("g1"));
    let expected = groups(&[("g1", "a", &["a"])]);
    for client in [&mut a, &mut b, &mut c] {
        assert_eq!(client.drain(), std::slice::from_ref(&expected));
    }
    idle.assert_silent();
    h.send(&b, create("g1"));
    assert_eq!(b.drain(), [error(ErrorKind::GroupExists)]);
    a.assert_silent();
    c.assert_silent();
}

#[test]
fn joining_is_explicit_sorted_and_idempotent() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&b, create("g1"));
    a.drain();
    b.drain();
    c.drain();
    h.send(&a, join_group("g1"));
    let expected = groups(&[("g1", "b", &["a", "b"])]);
    for client in [&mut a, &mut b, &mut c] {
        assert_eq!(client.drain(), std::slice::from_ref(&expected));
    }
    h.send(&a, join_group("g1"));
    a.assert_silent();
    b.assert_silent();
    c.assert_silent();
}

#[test]
fn list_groups_is_sorted_and_only_replies_to_requester() {
    let mut h = Harness::default();
    let [mut a, mut b] = h.join(["a", "b"]);
    h.send(&a, create("z"));
    h.send(&b, create("a"));
    a.drain();
    b.drain();
    h.send(&a, ClientMsg::ListGroups);
    assert_eq!(
        a.drain(),
        [groups(&[("a", "b", &["b"]), ("z", "a", &["a"])])]
    );
    b.assert_silent();
    let mut newcomer = h.register("new");
    newcomer.drain();
    h.send(&newcomer, ClientMsg::ListGroups);
    assert_eq!(
        newcomer.drain(),
        [groups(&[("a", "b", &["b"]), ("z", "a", &["a"])])]
    );
}

#[test]
fn messages_reach_only_members_including_sender() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    a.drain();
    b.drain();
    c.drain();
    h.send(&a, say("g1", "hello"));
    let msg = ServerMsg::GroupMsg {
        id: 1,
        group: "g1".into(),
        from: "a".into(),
        text: "hello".into(),
    };
    assert_eq!(a.drain(), std::slice::from_ref(&msg));
    assert_eq!(b.drain(), [msg]);
    c.assert_silent();
    h.send(&c, say("g1", "intrusion"));
    assert_eq!(c.drain(), [error(ErrorKind::NotMember)]);
    a.assert_silent();
    b.assert_silent();
}

#[test]
fn unknown_groups_and_unregistered_commands_are_rejected() {
    let mut h = Harness::default();
    let [mut a] = h.join(["a"]);
    for msg in [
        join_group("missing"),
        leave("missing"),
        delete("missing"),
        say("missing", "hi"),
    ] {
        h.send(&a, msg);
        assert_eq!(a.drain(), [error(ErrorKind::UnknownGroup)]);
    }
    let mut idle = h.connect();
    for msg in [
        create("g1"),
        join_group("g1"),
        leave("g1"),
        delete("g1"),
        ClientMsg::ListGroups,
    ] {
        h.send(&idle, msg);
        assert_eq!(idle.drain(), [error(ErrorKind::NotRegistered)]);
    }
}

#[test]
fn disconnect_removes_all_memberships_deletes_empty_groups_and_allows_reuse() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, create("shared"));
    h.send(&a, create("alone"));
    h.send(&b, join_group("shared"));
    a.drain();
    b.drain();
    c.drain();
    h.disconnect(&a);
    let expected = [users(&["b", "c"]), groups(&[("shared", "a", &["b"])])];
    assert_eq!(b.drain(), expected);
    assert_eq!(c.drain(), expected);
    let mut a2 = h.register("a");
    a2.drain();
    b.drain();
    c.drain();
    h.send(&a2, say("shared", "hi"));
    assert_eq!(a2.drain(), [error(ErrorKind::NotMember)]);
    h.disconnect(&b);
    assert_eq!(c.drain(), [users(&["a", "c"]), groups(&[])]);
    a2.drain();
    h.send(&a2, create("shared"));
    assert_eq!(a2.drain(), [groups(&[("shared", "a", &["a"])])]);
}

#[test]
fn leaving_updates_everyone_closes_membership_and_deletes_last_member_group() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    a.drain();
    b.drain();
    c.drain();
    h.send(&c, leave("g1"));
    assert_eq!(c.drain(), [error(ErrorKind::NotMember)]);
    h.send(&a, leave("g1"));
    let expected = groups(&[("g1", "a", &["b"])]);
    for client in [&mut a, &mut b, &mut c] {
        assert_eq!(client.drain(), std::slice::from_ref(&expected));
    }
    h.send(&a, say("g1", "hi"));
    assert_eq!(a.drain(), [error(ErrorKind::NotMember)]);
    h.send(&b, leave("g1"));
    for client in [&mut a, &mut b, &mut c] {
        assert_eq!(client.drain(), [groups(&[])]);
    }
    h.send(&c, create("g1"));
    assert_eq!(c.drain(), [groups(&[("g1", "c", &["c"])])]);
}

#[test]
fn only_the_creator_can_delete_and_everyone_is_told() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    a.drain();
    b.drain();
    c.drain();

    h.send(&b, delete("g1"));
    assert_eq!(b.drain(), [error(ErrorKind::NotCreator)]);
    h.send(&c, delete("g1"));
    assert_eq!(c.drain(), [error(ErrorKind::NotCreator)]);
    a.assert_silent();

    h.send(&a, delete("g1"));
    // members hear GroupDeleted before the new list; a non-member only gets the list
    for client in [&mut a, &mut b] {
        assert_eq!(client.drain(), [group_deleted("g1"), groups(&[])]);
    }
    assert_eq!(c.drain(), [groups(&[])]);

    h.send(&b, say("g1", "hi"));
    assert_eq!(b.drain(), [error(ErrorKind::UnknownGroup)]);
    h.send(&b, create("g1"));
    for client in [&mut a, &mut b, &mut c] {
        assert_eq!(client.drain(), [groups(&[("g1", "b", &["b"])])]);
    }
}

#[test]
fn creator_can_delete_after_leaving() {
    let mut h = Harness::default();
    let [mut a, mut b] = h.join(["a", "b"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    h.send(&a, leave("g1"));
    a.drain();
    b.drain();
    h.send(&a, delete("g1"));
    // B was a member and A is the creator; both are told
    assert_eq!(a.drain(), [group_deleted("g1"), groups(&[])]);
    assert_eq!(b.drain(), [group_deleted("g1"), groups(&[])]);
}

// --- Typing ---

#[test]
fn group_typing_reaches_other_members_only() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c, mut d] = h.join(["a", "b", "c", "d"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    h.send(&c, join_group("g1"));
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.drain();
    }
    h.send(&a, typing_group("g1"));
    let expected = got_typing_group("g1", "a");
    assert_eq!(b.drain(), std::slice::from_ref(&expected));
    assert_eq!(c.drain(), std::slice::from_ref(&expected));
    a.assert_silent(); // never your own indicator
    d.assert_silent(); // non-member
}

#[test]
fn group_typing_from_non_member_reaches_nobody_and_is_not_an_error() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    for client in [&mut a, &mut b, &mut c] {
        client.drain();
    }
    h.send(&c, typing_group("g1"));
    a.assert_silent();
    b.assert_silent();
    c.assert_silent();
}

#[test]
fn typing_after_leaving_a_group_is_dropped() {
    let mut h = Harness::default();
    let [mut a, mut b] = h.join(["a", "b"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    h.send(&b, leave("g1"));
    a.drain();
    b.drain();
    h.send(&b, typing_group("g1"));
    h.send(&a, typing_group("g1"));
    a.assert_silent();
    b.assert_silent();
}

#[test]
fn typing_in_unknown_group_is_silent() {
    let mut h = Harness::default();
    let [mut a] = h.join(["a"]);
    h.send(&a, typing_group("nope"));
    a.assert_silent();
}

#[test]
fn dm_typing_reaches_only_the_target_and_names_the_sender() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, typing_dm("b"));
    assert_eq!(b.drain(), [got_typing_dm("a")]);
    a.assert_silent();
    c.assert_silent();
}

#[test]
fn dm_typing_to_offline_user_or_self_is_silent() {
    let mut h = Harness::default();
    let [mut a, b] = h.join(["a", "b"]);
    h.disconnect(&b);
    a.drain();
    h.send(&a, typing_dm("b"));
    h.send(&a, typing_dm("zed"));
    h.send(&a, typing_dm("a"));
    a.assert_silent();
}

#[test]
fn typing_before_register_is_rejected() {
    let mut h = Harness::default();
    let mut idle = h.connect();
    h.send(&idle, typing_dm("b"));
    assert_eq!(idle.drain(), [error(ErrorKind::NotRegistered)]);
}

// --- Bots (F10) ---

#[test]
fn created_bot_appears_in_group_list() {
    let mut h = Harness::default();
    let [a, mut b] = h.join(["a", "b"]);
    h.send(&a, create("g1"));
    let bot = ClientMsg::CreateBot {
        group: "g1".into(),
        name: "Pirate".into(),
        personality: "You talk like a pirate.".into(),
    };
    h.send(&a, bot);
    let Some(ServerMsg::Groups { groups }) = b.drain().pop() else {
        panic!("expected a Groups broadcast");
    };
    assert_eq!(groups[0].bots, ["Pirate"]);
}

/// "a" creates g1 with bot Pirate; "b" joins; "c" stays outside.
fn bot_group(h: &mut Harness) -> [super::testkit::Client; 3] {
    let [a, b, c] = h.join(["a", "b", "c"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    let bot = ClientMsg::CreateBot {
        group: "g1".into(),
        name: "Pirate".into(),
        personality: "Talk like a pirate.".into(),
    };
    h.send(&a, bot);
    [a, b, c]
}

#[test]
fn only_a_mention_asks_the_bot() {
    let mut h = Harness::default();
    let [a, b, _c] = bot_group(&mut h);
    h.send(&a, say("g1", "hello"));
    assert!(h.ai_jobs().is_empty());
    h.send(&b, say("g1", "@Pirate hi"));
    let jobs = h.ai_jobs();
    assert_eq!(jobs.len(), 1);
    assert!(jobs[0].system.contains("Talk like a pirate."));
    assert!(jobs[0].prompt.contains("a: hello\nb: @Pirate hi"));
}

#[test]
fn bot_reply_reaches_members_only() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = bot_group(&mut h);
    a.drain();
    b.drain();
    c.drain();
    h.apply(Event::AiReply {
        group: "g1".into(),
        bot: "Pirate".into(),
        text: "Arr!".into(),
    });
    let reply = ServerMsg::GroupMsg {
        id: 1,
        group: "g1".into(),
        from: "Pirate".into(),
        text: "Arr!".into(),
    };
    assert_eq!(a.drain(), vec![reply.clone()]);
    assert_eq!(b.drain(), [reply]);
    c.assert_silent();
}

// --- Message ids ---

fn msg_ids(msgs: Vec<ServerMsg>) -> Vec<u64> {
    let ids = msgs.into_iter().filter_map(|m| match m {
        ServerMsg::Dm { id, .. } | ServerMsg::GroupMsg { id, .. } => Some(id),
        _ => None,
    });
    ids.collect()
}

#[test]
fn ids_are_one_sequence_across_dms_groups_and_bot_replies() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = bot_group(&mut h);
    for client in [&mut a, &mut b, &mut c] {
        client.drain();
    }
    h.send(&a, dm("c", "one"));
    h.send(&a, say("g1", "two"));
    h.send(&b, say("g1", "@Pirate hi"));
    let job = h.ai_jobs().pop().unwrap();
    h.apply(Event::AiReply {
        group: job.group,
        bot: job.bot,
        text: "Ahoy".into(),
    });
    h.send(&a, dm("b", "five"));
    assert_eq!(msg_ids(a.drain()), [1, 2, 3, 4, 5]);
}

#[test]
fn rejected_messages_consume_no_id() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, create("g1"));
    for client in [&mut a, &mut b, &mut c] {
        client.drain();
    }
    let idle = h.connect();
    h.send(&idle, dm("a", "rejected")); // not registered
    h.send(&a, dm("zed", "rejected")); // unknown user
    h.send(&a, say("missing", "rejected")); // unknown group
    h.send(&c, say("g1", "rejected")); // not a member
    h.send(&a, dm("b", "ok"));
    assert_eq!(b.drain(), [got_dm("a", "b", "ok", 1)]);
}

// --- Read receipts ---

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
fn dm_receipt_names_the_reader_as_peer_and_never_echoes() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c] = h.join(["a", "b", "c"]);
    h.send(&a, dm("b", "hi"));
    a.drain();
    b.drain();
    h.send(&b, read(dm_room("a"), 1));
    // `room` names the other party, so A sees the room as "b"
    let receipt = ServerMsg::Read {
        room: dm_room("b"),
        by: "b".into(),
        upto: 1,
    };
    assert_eq!(a.drain(), [receipt]);
    b.assert_silent(); // never echoed
    c.assert_silent();
    // self-DM, offline peer, unknown group: silently ignored
    for room in [dm_room("b"), dm_room("offline"), group_room("missing")] {
        h.send(&b, read(room, 1));
    }
    for client in [&mut a, &mut b, &mut c] {
        client.assert_silent();
    }
    // unregistered reader
    let mut idle = h.connect();
    h.send(&idle, read(dm_room("a"), 1));
    idle.assert_silent();
    a.assert_silent();
}

#[test]
fn group_receipts_reach_other_members_only_and_invalid_readers_are_silent() {
    let mut h = Harness::default();
    let [mut a, mut b, mut c, mut d] = h.join(["a", "b", "c", "d"]);
    h.send(&a, create("g1"));
    h.send(&b, join_group("g1"));
    h.send(&c, join_group("g1"));
    h.send(&a, say("g1", "hi"));
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.drain();
    }
    h.send(&b, read(group_room("g1"), 1));
    let expected = ServerMsg::Read {
        room: group_room("g1"),
        by: "b".into(),
        upto: 1,
    };
    assert_eq!(a.drain(), std::slice::from_ref(&expected));
    assert_eq!(c.drain(), [expected]);
    b.assert_silent(); // never echoed
    d.assert_silent(); // non-member
    h.send(&d, read(group_room("g1"), 1)); // non-member reader
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.assert_silent();
    }
    h.send(&b, leave("g1"));
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.drain();
    }
    h.send(&b, read(group_room("g1"), 1)); // left the group
    for client in [&mut a, &mut b, &mut c, &mut d] {
        client.assert_silent();
    }
}
