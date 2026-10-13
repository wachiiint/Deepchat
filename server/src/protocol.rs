//! Wire messages. Variant order matches `docs/protocol.md` (postcard encodes by index).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMsg {
    Register { name: String },
    Dm { to: String, text: String },
    CreateGroup { group: String },
    ListGroups,
    JoinGroup { group: String },
    GroupMsg { group: String, text: String },
    Pong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome { name: String },
    Users { names: Vec<String> },
    Dm { from: String, to: String, text: String },
    Groups { groups: Vec<GroupInfo> },
    GroupMsg { group: String, from: String, text: String },
    Ping,
    Error { kind: ErrorKind },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group: String,
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
    fn round_trip() {
        let c = ClientMsg::Dm { to: "bob".into(), text: "hi".into() };
        let bytes = postcard::to_stdvec(&c).unwrap();
        assert_eq!(postcard::from_bytes::<ClientMsg>(&bytes).unwrap(), c);

        let s = ServerMsg::Groups {
            groups: vec![GroupInfo { group: "net".into(), members: vec!["alice".into()] }],
        };
        let bytes = postcard::to_stdvec(&s).unwrap();
        assert_eq!(postcard::from_bytes::<ServerMsg>(&bytes).unwrap(), s);
    }
}
