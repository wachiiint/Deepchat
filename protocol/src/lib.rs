//! Wire protocol shared by the server and the client. See `docs/protocol.md`.

pub mod framer;
pub mod messages;

pub use framer::{FrameError, MAX_FRAME, read_frame, write_frame};
pub use messages::{ClientMsg, ErrorKind, GroupInfo, ServerMsg};
