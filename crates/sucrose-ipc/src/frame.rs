//! Length-prefixed frames: a `u32` little-endian byte count, then that many bytes of postcard.
//!
//! postcard values are not self-delimiting on a stream, so every message is wrapped in a frame. The length is checked
//! against [`MAX_FRAME`] before anything is allocated, so a peer cannot make the reader reserve memory by lying.

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use crate::IpcError;

/// Largest frame payload either side accepts, in bytes.
pub const MAX_FRAME: u32 = 64 * 1024;

/// Writes `message` as one frame and flushes.
pub fn write_frame<W: Write, T: Serialize>(writer: &mut W, message: &T) -> Result<(), IpcError> {
    let payload = postcard::to_stdvec(message).map_err(IpcError::Encode)?;
    let length = u32::try_from(payload.len()).map_err(|_| IpcError::FrameTooLarge(u32::MAX))?;
    if length > MAX_FRAME {
        return Err(IpcError::FrameTooLarge(length));
    }
    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

/// Reads one frame and decodes it.
///
/// Returns [`IpcError::Closed`] when the peer is gone at a frame boundary (or in the middle of one),
/// [`IpcError::FrameTooLarge`] when the header announces more than [`MAX_FRAME`] bytes (nothing further is read), and
/// [`IpcError::Decode`] when the payload is not a valid `T`.
pub fn read_frame<R: Read, T: DeserializeOwned>(reader: &mut R) -> Result<T, IpcError> {
    let mut header = [0_u8; 4];
    reader.read_exact(&mut header)?;
    let length = u32::from_le_bytes(header);
    if length > MAX_FRAME {
        return Err(IpcError::FrameTooLarge(length));
    }
    // The cast cannot truncate: `length` is at most MAX_FRAME here.
    let mut payload = vec![0_u8; length as usize];
    reader.read_exact(&mut payload)?;
    postcard::from_bytes(&payload).map_err(IpcError::Decode)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::protocol::{PROTOCOL_VERSION, ToRenderer, ToService};

    fn hello() -> ToService {
        ToService::Hello {
            protocol_version: PROTOCOL_VERSION,
            pid: 4242,
            slot: 3,
        }
    }

    #[test]
    fn a_written_frame_reads_back_equal() {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &hello()).expect("write Hello");
        write_frame(&mut bytes, &ToRenderer::Close).expect("write Close");

        let mut reader = Cursor::new(bytes);
        assert_eq!(read_frame::<_, ToService>(&mut reader).expect("read Hello"), hello());
        assert_eq!(
            read_frame::<_, ToRenderer>(&mut reader).expect("read Close"),
            ToRenderer::Close
        );
    }

    #[test]
    fn a_header_above_the_limit_is_refused_without_reading_further() {
        let header = (MAX_FRAME + 1).to_le_bytes();
        let mut reader = Cursor::new(header.to_vec());

        let error = read_frame::<_, ToService>(&mut reader).expect_err("oversized frame");

        assert!(
            matches!(error, IpcError::FrameTooLarge(n) if n == MAX_FRAME + 1),
            "{error:?}"
        );
        assert_eq!(reader.position(), 4, "only the header may be consumed");
    }

    #[test]
    fn a_message_above_the_limit_is_not_written() {
        let mut bytes = Vec::new();
        let error = write_frame(&mut bytes, &vec![0_u8; MAX_FRAME as usize + 1]).expect_err("oversized message");

        assert!(matches!(error, IpcError::FrameTooLarge(_)), "{error:?}");
        assert!(bytes.is_empty(), "nothing may reach the stream");
    }

    #[test]
    fn bytes_that_are_not_a_message_fail_to_decode() {
        let mut bytes = 8_u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0xFF; 8]);

        let error = read_frame::<_, ToService>(&mut Cursor::new(bytes)).expect_err("garbage payload");

        assert!(matches!(error, IpcError::Decode(_)), "{error:?}");
    }

    #[test]
    fn a_stream_that_ends_inside_a_frame_is_closed() {
        let mut bytes = 8_u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[1, 2]);

        let error = read_frame::<_, ToService>(&mut Cursor::new(bytes)).expect_err("truncated frame");

        assert!(matches!(error, IpcError::Closed), "{error:?}");
    }
}
