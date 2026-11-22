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
