use postcard::{from_bytes, to_stdvec};
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt, io};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const MAX_FRAME: u32 = 64 * 1024;

// error for this module
#[derive(Debug)]
pub enum FrameError {
    ErrorLenOutofScope(u32),
    ErrorSerializationFailed(postcard::Error),
    ErrorDeSerializationFailed(postcard::Error),
    ErrorWriteFail(io::Error),
    ErrorReadFail(io::Error),
}
// good error for display
impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            FrameError::ErrorSerializationFailed(e) => write!(f, "Serialized fail: {e}"),
            FrameError::ErrorDeSerializationFailed(e) => write!(f, "Deserialized fail: {e}"),
            FrameError::ErrorReadFail(e) => write!(f, "Read error: {e}"),
            FrameError::ErrorWriteFail(e) => write!(f, "Write error: {e}"),
            FrameError::ErrorLenOutofScope(e) => write!(f, "msg len = {e} > {MAX_FRAME}"),
        }
    }
}

// write function, W = trait for TCP, file, stdout / T = msg's dtype
pub async fn write_frame<W, T>(w: &mut W, msg: &T) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let body: Vec<u8> = to_stdvec(msg).map_err(FrameError::ErrorSerializationFailed)?;
    if body.len() > MAX_FRAME as usize {
        return Err(FrameError::ErrorLenOutofScope(body.len() as u32));
    };
    let x: [u8; 4] = (body.len() as u32).to_be_bytes();
    let frame: Vec<u8> = x.into_iter().chain(body).collect(); // concat array + vector in performance way
    w.write_all(&frame)
        .await
        .map_err(FrameError::ErrorWriteFail) // write frame into stream
}

pub async fn read_frame<R, T>(r: &mut R) -> Result<Option<T>, FrameError>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    // read first u32 len
    let n: u32 = match r.read_u32().await {
        Ok(n) => n,
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None), //Expected disconnect
        Err(e) => return Err(FrameError::ErrorReadFail(e)),
    };
    if n > MAX_FRAME {
        return Err(FrameError::ErrorLenOutofScope(n));
    }
    let mut buf: Vec<u8> = vec![0; n as usize]; // place holder zero vector
    r.read_exact(&mut buf) // read from stream
        .await
        .map_err(FrameError::ErrorReadFail)?;
    let out: T = from_bytes(&buf).map_err(FrameError::ErrorDeSerializationFailed)?; //decode into buffer
    Ok(Some(out)) //return
}

// TEST
#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::ClientMsg;
    use tokio::io::duplex;

    // two frames back to back must come out as two separate messages
    #[tokio::test]
    async fn round_trip() {
        let (mut a, mut b) = duplex(1024);
        let m1 = ClientMsg::Register {
            name: "alice".into(),
        };
        let m2 = ClientMsg::Dm {
            to: "bob".into(),
            text: "hi".into(),
        };
        write_frame(&mut a, &m1).await.unwrap();
        write_frame(&mut a, &m2).await.unwrap();
        assert_eq!(read_frame::<_, ClientMsg>(&mut b).await.unwrap(), Some(m1));
        assert_eq!(read_frame::<_, ClientMsg>(&mut b).await.unwrap(), Some(m2));
    }

    #[tokio::test]
    async fn disconnect_is_none() {
        let (a, mut b) = duplex(1024);
        drop(a);
        assert_eq!(read_frame::<_, ClientMsg>(&mut b).await.unwrap(), None);
    }

    #[tokio::test]
    async fn rejects_oversized_len() {
        let (mut a, mut b) = duplex(1024);
        a.write_u32(MAX_FRAME + 1).await.unwrap();
        let got = read_frame::<_, ClientMsg>(&mut b).await;
        assert!(matches!(got, Err(FrameError::ErrorLenOutofScope(_))));
    }
}
