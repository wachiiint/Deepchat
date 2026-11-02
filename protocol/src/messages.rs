//! Wire messages. Variant order matches `docs/protocol.md` (postcard encodes by index).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMsg {
    Pong,
    Register { name: String },
    Dm { to: String, text: String },
    CreateGroup { group: String },
    ListGroups,
    JoinGroup { group: String },
    GroupMsg { group: String, text: String },
    LeaveGroup { group: String },
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
        from: String,
        to: String,
        text: String,
    },
    Groups {
        groups: Vec<GroupInfo>,
    },
    GroupMsg {
        group: String,
        from: String,
        text: String,
    },
    Error {
        kind: ErrorKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group: String,
    pub creator: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    NameTaken,
    NotRegistered,
    UnknownUser,
    GroupExists,
    UnknownGroup,
    NotMember,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_metadata_round_trips_and_leave_preserves_existing_variant_indexes() {
        let groups = ServerMsg::Groups {
            groups: vec![GroupInfo {
                group: "study".into(),
                creator: "alice".into(),
                members: vec!["alice".into(), "bob".into()],
            }],
        };
        let bytes = postcard::to_stdvec(&groups).unwrap();
        assert_eq!(postcard::from_bytes::<ServerMsg>(&bytes).unwrap(), groups);
        assert_eq!(postcard::to_stdvec(&ClientMsg::Pong).unwrap(), [6]);
        let leave = ClientMsg::LeaveGroup {
            group: "study".into(),
        };
        let bytes = postcard::to_stdvec(&leave).unwrap();
        assert_eq!(bytes[0], 7);
        assert_eq!(postcard::from_bytes::<ClientMsg>(&bytes).unwrap(), leave);
    }
}
