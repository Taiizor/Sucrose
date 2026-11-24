//! The local channel between the Sucrose service and its renderers (D-009, D-021).
//!
//! One blocking local socket per user: a named pipe on Windows, a Unix socket elsewhere. The service binds the
//! endpoint first, which doubles as the single-instance check (D-023): a second bind on the same name fails with
//! [`IpcError::AlreadyRunning`]. Messages are postcard values in length-prefixed [`frame`]s, with the types in
//! [`protocol`]. There is no async runtime (D-012): a connection is read on a thread of its own.
//!
//! The same endpoint serves the Portal: the first message of a connection tells the service which kind of peer it is
//! (D-033), and the Portal's own messages live in [`portal`].

pub mod frame;
pub mod portal;
pub mod protocol;

use std::{ffi::OsStr, io};

use interprocess::{
    TryClone,
    local_socket::{
        GenericFilePath, GenericNamespaced, ListenerOptions, Name, NameType, Stream, ToFsName, ToNsName,
        traits::{Listener as _, Stream as _},
    },
};
use serde::{Serialize, de::DeserializeOwned};
use sucrose_core::paths::DATA_DIR_ENV;
use sucrose_platform::PlatformError;

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
    /// The OS cannot say which user runs this process, so the endpoint cannot be named.
    #[error("cannot identify the current user: {0}")]
    Identity(String),
    /// This OS does not provide the operation.
    #[error("{0} is not supported on this OS")]
    Unsupported(&'static str),
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

/// Where one user's service listens, and whom the endpoint admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// The endpoint name, shared by the service, its renderers and the Portal.
    pub name: String,
    /// The account the endpoint is restricted to, where the OS can restrict it (the SID on Windows).
    pub owner: Option<String>,
}

/// The endpoint of the current user, named for the data folder in `SUCROSE_DATA_DIR` when that is set.
///
/// The account SID names and owns it on Windows; the user name is only the fallback where the platform has no user
/// identity (macOS and Linux, which have no wallpaper yet).
pub fn service_endpoint() -> Result<Endpoint, IpcError> {
    service_endpoint_for(std::env::var_os(DATA_DIR_ENV).as_deref())
}

/// [`service_endpoint`] for an explicit data-folder override, so a test can name the endpoint of a service it started
/// on a scratch folder without touching its own environment.
pub fn service_endpoint_for(data_dir_override: Option<&OsStr>) -> Result<Endpoint, IpcError> {
    match sucrose_platform::current().user_key() {
        Ok(key) => Ok(Endpoint {
            name: endpoint_name_for(&key, data_dir_override),
            owner: Some(key),
        }),
        Err(PlatformError::Unsupported(_)) => {
            let user = std::env::var("USERNAME")
                .or_else(|_| std::env::var("USER"))
                .unwrap_or_else(|_| "user".to_owned());
            Ok(Endpoint {
                name: endpoint_name_for(&user, data_dir_override),
                owner: None,
            })
        }
        Err(error) => Err(IpcError::Identity(error.to_string())),
    }
}

/// The endpoint name for a user and an optional data-folder override: [`endpoint_name`] when the override is unset or
/// empty, else that name followed by the FNV-1a 64 hash of the folder text as 16 hex digits. The hash, not the path,
/// goes into the name, so a scratch run never meets the user's own service and the folder never reaches the OS-wide
/// name (T-03-16).
pub fn endpoint_name_for(user_key: &str, data_dir_override: Option<&OsStr>) -> String {
    match data_dir_override {
        Some(folder) if !folder.is_empty() => {
            format!(
                "{}.{:016x}",
                endpoint_name(user_key),
                fnv1a64(folder.to_string_lossy().as_bytes())
            )
        }
        _ => endpoint_name(user_key),
    }
}

/// FNV-1a over 64 bits: a stable, dependency-free hash (the standard library's hashers may change between releases).
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
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

    /// Ends the connection for every handle of it: a reader blocked in [`Connection::recv`] on another handle
    /// returns an error and the peer's next `recv` returns [`IpcError::Closed`]. Only the service side of a
    /// connection (the end `Listener::accept` returned) can be disconnected.
    pub fn disconnect(&self) -> Result<(), IpcError> {
        Err(IpcError::Unsupported("disconnecting a connection"))
    }

    /// Another handle to the same connection, so one thread can read while another writes.
    pub fn try_clone(&self) -> Result<Self, IpcError> {
        Ok(Self {
            stream: self.stream.try_clone()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_matches_the_published_vectors() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn the_endpoint_name_is_unchanged_without_a_data_folder_override() {
        assert_eq!(endpoint_name_for("S-1-5-21", None), "Sucrose.Service.S-1-5-21");
        assert_eq!(
            endpoint_name_for("S-1-5-21", Some(OsStr::new(""))),
            "Sucrose.Service.S-1-5-21"
        );
    }

    #[test]
    fn a_data_folder_override_adds_sixteen_hex_digits_and_never_the_path() {
        let name = endpoint_name_for("S-1-5-21", Some(OsStr::new("E:\\scratch\\one")));

        let suffix = name
            .strip_prefix("Sucrose.Service.S-1-5-21.")
            .expect("the name keeps its prefix");
        assert_eq!(suffix.len(), 16);
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!name.contains("scratch"));
    }

    #[test]
    fn the_same_folder_gives_the_same_name_and_another_folder_another() {
        let one = endpoint_name_for("k", Some(OsStr::new("a/one")));

        assert_eq!(one, endpoint_name_for("k", Some(OsStr::new("a/one"))));
        assert_ne!(one, endpoint_name_for("k", Some(OsStr::new("a/two"))));
        assert_ne!(one, endpoint_name_for("k", None));
    }

    #[test]
    fn service_endpoint_for_names_the_override() {
        let plain = service_endpoint_for(None).expect("endpoint");
        let scratch = service_endpoint_for(Some(OsStr::new("scratch"))).expect("endpoint");

        assert_eq!(plain.owner, scratch.owner);
        assert!(scratch.name.starts_with(&plain.name));
        assert_ne!(plain.name, scratch.name);
    }
}
