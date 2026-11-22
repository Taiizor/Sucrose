//! The local channel between the Sucrose service and its renderers (D-009, D-021).
//!
//! One blocking local socket per user: a named pipe on Windows, a Unix socket elsewhere. The service binds the
//! endpoint first, which doubles as the single-instance check (D-023): a second bind on the same name fails with
//! [`IpcError::AlreadyRunning`]. Messages are postcard values in length-prefixed [`frame`]s, with the types in
//! [`protocol`]. There is no async runtime (D-012): a connection is read on a thread of its own.

pub mod frame;
pub mod protocol;

use std::io;

use interprocess::{
    TryClone,
    local_socket::{
        GenericFilePath, GenericNamespaced, ListenerOptions, Name, NameType, Stream, ToFsName, ToNsName,
        traits::{Listener as _, Stream as _},
    },
};
use serde::{Serialize, de::DeserializeOwned};

/// Why a channel operation failed.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// Another service already owns the endpoint.
    #[error("another Sucrose already owns the endpoint")]
    AlreadyRunning,
    /// A frame is longer than [`frame::MAX_FRAME`].
    #[error("a frame of {0} bytes exceeds the limit")]
    FrameTooLarge(u32),
    /// A frame arrived intact but its payload is not a valid message.
    #[error("the frame payload cannot be decoded: {0}")]
    Decode(postcard::Error),
    /// A message cannot be encoded.
    #[error("the message cannot be encoded: {0}")]
    Encode(postcard::Error),
    /// The socket failed.
    #[error("the local socket failed: {0}")]
    Io(io::Error),
    /// The peer closed the connection.
    #[error("the connection is closed")]
    Closed,
}

impl From<io::Error> for IpcError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset => Self::Closed,
            _ => Self::Io(error),
        }
    }
}

/// The endpoint name of the service of one user: `Sucrose.Service.<user_key>`.
///
/// The key must identify the user (the account SID on Windows), so two users on one machine never share a service.
pub fn endpoint_name(user_key: &str) -> String {
    format!("Sucrose.Service.{user_key}")
}

/// The socket name for an endpoint: a namespaced name where the OS has one (Windows pipes), else a file in the
/// temporary folder (macOS).
fn socket_name(endpoint: &str) -> io::Result<Name<'static>> {
    if GenericNamespaced::is_supported() {
        endpoint.to_owned().to_ns_name::<GenericNamespaced>()
    } else {
        std::env::temp_dir()
            .join(format!("{endpoint}.sock"))
            .to_fs_name::<GenericFilePath>()
    }
}

/// The service side of the channel: owns the endpoint and accepts renderer connections.
#[derive(Debug)]
pub struct Listener {
    inner: interprocess::local_socket::Listener,
}

impl Listener {
    /// Binds the endpoint `name`. `owner_sid` restricts access to that account on Windows.
    ///
    /// Fails with [`IpcError::AlreadyRunning`] when the endpoint already exists.
    pub fn bind(name: &str, owner_sid: Option<&str>) -> Result<Self, IpcError> {
        let options = ListenerOptions::new().name(socket_name(name)?);
        #[cfg(windows)]
        let options = match owner_sid {
            Some(sid) => restrict_to_owner(options, sid)?,
            None => options,
        };
        #[cfg(not(windows))]
        let _ = owner_sid;
        match options.create_sync() {
            Ok(inner) => Ok(Self { inner }),
            Err(error) if is_endpoint_taken(&error) => Err(IpcError::AlreadyRunning),
            Err(error) => Err(IpcError::Io(error)),
        }
    }

    /// Waits for the next connection.
    pub fn accept(&self) -> Result<Connection, IpcError> {
        Ok(Connection {
            stream: self.inner.accept()?,
        })
    }
}

/// Limits the pipe to one account: a protected DACL with a single entry granting generic-all to that SID, so
/// other users, groups and even elevated processes of other accounts cannot open it.
#[cfg(windows)]
fn restrict_to_owner<'n>(options: ListenerOptions<'n>, sid: &str) -> Result<ListenerOptions<'n>, IpcError> {
    use interprocess::os::windows::{local_socket::ListenerOptionsExt, security_descriptor::SecurityDescriptor};

    let invalid = |message: &str| IpcError::Io(io::Error::new(io::ErrorKind::InvalidInput, message.to_owned()));
    // A SID string is only ever letters S, digits and dashes; anything else must not reach the SDDL text.
    if sid.is_empty() || !sid.chars().all(|c| c == 'S' || c == '-' || c.is_ascii_digit()) {
        return Err(invalid("the owner SID is not a valid SID string"));
    }
    let sddl = widestring::U16CString::from_str(format!("D:P(A;;GA;;;{sid})"))
        .map_err(|_| invalid("the security descriptor text contains a NUL"))?;
    let descriptor = SecurityDescriptor::deserialize(&sddl)?;
    Ok(options.security_descriptor(descriptor))
}

/// Whether a bind error means "the endpoint already exists".
fn is_endpoint_taken(error: &io::Error) -> bool {
    matches!(error.kind(), io::ErrorKind::AddrInUse | io::ErrorKind::PermissionDenied)
}

/// Connects to the service at the endpoint `name`.
pub fn connect(name: &str) -> Result<Connection, IpcError> {
    Ok(Connection {
        stream: Stream::connect(socket_name(name)?)?,
    })
}

/// One end of an open channel: blocking, framed, typed.
#[derive(Debug)]
pub struct Connection {
    stream: Stream,
}

impl Connection {
    /// Sends one message.
    pub fn send<T: Serialize>(&mut self, message: &T) -> Result<(), IpcError> {
        frame::write_frame(&mut self.stream, message)
    }

    /// Waits for the next message.
    pub fn recv<T: DeserializeOwned>(&mut self) -> Result<T, IpcError> {
        frame::read_frame(&mut self.stream)
    }

    /// Another handle to the same connection, so one thread can read while another writes.
    pub fn try_clone(&self) -> Result<Self, IpcError> {
        Ok(Self {
            stream: self.stream.try_clone()?,
        })
    }
}
