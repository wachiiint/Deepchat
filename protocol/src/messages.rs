//! Wire messages. Variant order matches `docs/protocol.md` (postcard encodes by index).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMsg {
    Pong,
    Register {
        name: String,
    },
    Dm {
        to: String,
        text: String,
    },
    CreateGroup {
        group: String,
    },
    ListGroups,
    JoinGroup {
        group: String,
    },
    GroupMsg {
        group: String,
        text: String,
    },
    LeaveGroup {
        group: String,
    },
    DeleteGroup {
        group: String,
    },
    Typing {
        room: RoomRef,
    },
    CreateBot {
        group: String,
        name: String,
        personality: String,
    },
    RemoveBot {
        group: String,
        name: String,
    },
    Read {
        room: RoomRef,
        upto: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMsg {
    Ping,
    Welcome {
        name: String,
    },
    Users {
        names: Vec<String>,
    },
    Dm {
        id: u64,
        from: String,
        to: String,
        text: String,
    },
    Groups {
        groups: Vec<GroupInfo>,
    },
    GroupMsg {
        id: u64,
        group: String,
        from: String,
        text: String,
    },
    Error {
        kind: ErrorKind,
    },
    GroupDeleted {
        group: String,
    },
    Typing {
        room: RoomRef,
        from: String,
    },
    Read {
        room: RoomRef,
        by: String,
        upto: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group: String,
    pub creator: String,
    pub members: Vec<String>,
    pub bots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomRef {
    Dm { to: String },
    Group { group: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    NameTaken,
    NotRegistered,
    UnknownUser,
    GroupExists,
    UnknownGroup,
    NotMember,
    NotCreator,
    BotLimit,
    BotNameTaken,
}

#[cfg(test)]
mod read_tests {
    use super::*;
    #[test]
    fn read_receipts_append_to_main_and_preserve_exact_ids() {
        for room in [
            RoomRef::Dm { to: "alice".into() },
            RoomRef::Group { group: "g".into() },
        ] {
            let msg = ClientMsg::Read {
                room: room.clone(),
                upto: u64::MAX,
            };
            let bytes = postcard::to_stdvec(&msg).unwrap();
            assert_eq!(bytes[0], 12);
            assert_eq!(postcard::from_bytes::<ClientMsg>(&bytes).unwrap(), msg);
            let msg = ServerMsg::Read {
                room,
                by: "bob".into(),
                upto: u64::MAX,
            };
            let bytes = postcard::to_stdvec(&msg).unwrap();
            assert_eq!(bytes[0], 9);
            assert_eq!(postcard::from_bytes::<ServerMsg>(&bytes).unwrap(), msg);
        }
        assert_eq!(postcard::to_stdvec(&ClientMsg::Pong).unwrap(), [0]);
        assert_eq!(postcard::to_stdvec(&ServerMsg::Ping).unwrap(), [0]);
    }
}
