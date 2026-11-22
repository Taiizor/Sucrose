//! Finding and loading libmpv at runtime (D-017, D-018, D-020).
//!
//! The search is deliberately narrower than "the Windows DLL search path" (D-018): the folder of the executable
//! first, then the absolute entries of `PATH`. The current folder and relative or empty `PATH` entries are never
//! searched, which keeps a planted DLL out of the process (T-02-02). Every failure is a [`LoadError`] value with a hint
//! the tray can show; nothing here panics or exits.

use std::ffi::OsStr;
use std::fmt;
use std::os::raw::c_ulong;
use std::path::{Path, PathBuf};

use crate::sys;

/// File names libmpv has on this OS, in the order they are tried in every folder.
#[cfg(windows)]
pub const LIBRARY_NAMES: &[&str] = &["libmpv-2.dll", "mpv-2.dll"];
/// File names libmpv has on this OS, in the order they are tried in every folder.
#[cfg(target_os = "macos")]
pub const LIBRARY_NAMES: &[&str] = &["libmpv.2.dylib"];
/// File names libmpv has on this OS, in the order they are tried in every folder.
#[cfg(not(any(windows, target_os = "macos")))]
pub const LIBRARY_NAMES: &[&str] = &["libmpv.so.2"];

/// The client API major version the bindings were generated for; a different major is not ABI compatible.
const REQUIRED_MAJOR: u32 = 2;

/// A loaded libmpv: the resolved functions plus where they came from.
pub struct Mpv {
    /// The functions of the pinned `client.h`, resolved from the loaded library.
    pub api: sys::Mpv,
    /// The absolute path the library was loaded from.
    pub path: PathBuf,
    /// The `(major, minor)` client API version the library reports.
    pub client_api: (u32, u32),
}

impl fmt::Debug for Mpv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mpv")
            .field("path", &self.path)
            .field("client_api", &self.client_api)
            .finish_non_exhaustive()
    }
}

/// Why libmpv could not be located or loaded.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// No candidate file exists.
    #[error("libmpv was not found; looked for: {}", list(.searched))]
    NotFound {
        /// Every file that was checked, in search order.
        searched: Vec<PathBuf>,
    },
    /// A relative path was passed; libraries are only loaded by absolute path.
    #[error("refusing to load libmpv from the relative path {}", .path.display())]
    NotAbsolute {
        /// The rejected path.
        path: PathBuf,
    },
    /// The file is not loadable by this process (for example a 64-bit DLL in a 32-bit process).
    #[error("{} cannot be loaded by this process (wrong architecture or not a library)", .path.display())]
    WrongArchitecture {
        /// The file that was refused.
        path: PathBuf,
    },
    /// The OS refused to load the file, or it lacks a function the bindings need.
    #[error("cannot load {}: {}", .path.display(), describe(.source))]
    LoadFailed {
        /// The file that failed.
        path: PathBuf,
        /// The loader's error.
        source: libloading::Error,
    },
    /// The library reports a client API this build cannot use.
    #[error("libmpv client API {found:#x} does not match the required {required:#x} (the major version must be equal)")]
    ApiTooOld {
        /// The version the library reported (0 when it reports none).
        found: u32,
        /// The lowest accepted version.
        required: u32,
    },
}

impl LoadError {
    /// A sentence telling the user how to repair the problem.
    #[must_use]
    pub fn hint(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => {
                "libmpv was not found next to sucrose.exe or in a PATH folder. In a development build run \
                 `cargo xtask fetch-engines`; otherwise reinstall Sucrose to restore libmpv-2.dll next to sucrose.exe."
            }
            Self::NotAbsolute { .. } => "This is a bug in Sucrose: libmpv must be loaded by an absolute path.",
            Self::WrongArchitecture { .. } => {
                "The libmpv file was built for another processor architecture or is damaged. Run \
                 `cargo xtask fetch-engines --arch <arch>` in a development build, or reinstall Sucrose for this PC."
            }
            Self::LoadFailed { .. } => {
                "libmpv or one of the DLLs it needs could not be loaded. Reinstall Sucrose, and check the log for \
                 the Windows error."
            }
            Self::ApiTooOld { .. } => {
                "The libmpv file is too old or too new for this Sucrose. Run `cargo xtask fetch-engines` in a \
                 development build, or reinstall Sucrose."
            }
        }
    }
}

fn list(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The loader's own message plus the OS error behind it, when there is one.
fn describe(error: &libloading::Error) -> String {
    match std::error::Error::source(error) {
        Some(source) => format!("{error}: {source}"),
        None => error.to_string(),
    }
}

/// The folders that may hold libmpv, in search order: the exe folder, then each absolute `PATH` entry once.
/// Relative and empty entries are skipped, and so is the current folder (T-02-02).
fn folders(exe_dir: Option<&Path>, path_var: Option<&OsStr>) -> Vec<PathBuf> {
    let from_path = path_var.into_iter().flat_map(std::env::split_paths);
    let mut folders: Vec<PathBuf> = Vec::new();
    for folder in exe_dir.map(Path::to_path_buf).into_iter().chain(from_path) {
        if folder.as_os_str().is_empty() || !folder.is_absolute() {
            continue;
        }
        if !folders.iter().any(|known| same_folder(known, &folder)) {
            folders.push(folder);
        }
    }
    folders
}

/// Folder equality for de-duplication: trailing separators are ignored, and so is letter case on Windows.
fn same_folder(a: &Path, b: &Path) -> bool {
    let key = |path: &Path| {
        let text = path.to_string_lossy();
        let trimmed = text.trim_end_matches(['/', '\\']);
        if cfg!(windows) {
            trimmed.to_lowercase()
        } else {
            trimmed.to_owned()
        }
    };
    key(a) == key(b)
}

fn files_in(folders: &[PathBuf]) -> Vec<PathBuf> {
    folders
        .iter()
        .flat_map(|folder| LIBRARY_NAMES.iter().map(move |name| folder.join(name)))
        .collect()
}

/// Every file that may hold libmpv: each name in the exe folder, then in each absolute `PATH` folder.
pub fn candidates(exe_dir: &Path, path_var: Option<&OsStr>) -> Vec<PathBuf> {
    files_in(&folders(Some(exe_dir), path_var))
}

/// The first candidate that exists, without loading it. The service may call this; only renderers load (D-017).
pub fn locate(exe_dir: &Path, path_var: Option<&OsStr>) -> Option<PathBuf> {
    candidates(exe_dir, path_var).into_iter().find(|file| file.is_file())
}

/// Like [`locate`], but a missing library is a [`LoadError::NotFound`] that lists everything searched.
fn resolve(exe_dir: Option<&Path>, path_var: Option<&OsStr>) -> Result<PathBuf, LoadError> {
    let searched = files_in(&folders(exe_dir, path_var));
    match searched.iter().find(|file| file.is_file()) {
        Some(found) => Ok(found.clone()),
        None => Err(LoadError::NotFound { searched }),
    }
}

/// Finds libmpv next to the running executable or on `PATH` and loads it.
///
/// Only the files named in [`LIBRARY_NAMES`] are considered, and never anything inside the C# app's install (D-018).
pub fn load_default() -> Result<Mpv, LoadError> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    if exe_dir.is_none() {
        tracing::warn!("cannot tell where sucrose.exe lives; searching PATH only for libmpv");
    }
    let path_var = std::env::var_os("PATH");
    let path = resolve(exe_dir.as_deref(), path_var.as_deref())?;
    load(&path)
}

/// Loads libmpv from an absolute path and checks its client API version.
pub fn load(path: &Path) -> Result<Mpv, LoadError> {
    if !path.is_absolute() {
        return Err(LoadError::NotAbsolute {
            path: path.to_path_buf(),
        });
    }
    let library = open(path)?;

    // An older or foreign library may lack the version function; that is reported as an API mismatch.
    type VersionFn = unsafe extern "C" fn() -> c_ulong;
    let required = REQUIRED_MAJOR << 16;
    // SAFETY: the name and the VersionFn signature come from the pinned client.h
    // (`unsigned long mpv_client_api_version(void)`); the function pointer is only used while `library` is alive.
    let reported = match unsafe { library.get::<VersionFn>(b"mpv_client_api_version\0") } {
        // SAFETY: the function takes no arguments and has no preconditions (client.h).
        Ok(version_fn) => version_u32(unsafe { version_fn() }),
        Err(_) => return Err(LoadError::ApiTooOld { found: 0, required }),
    };
    if reported >> 16 != REQUIRED_MAJOR {
        return Err(LoadError::ApiTooOld {
            found: reported,
            required,
        });
    }

    // SAFETY: every function pointer in `sys::Mpv` is resolved by the name and signature of the pinned client.h,
    // and the major version just checked guarantees the library keeps that ABI.
    let api = unsafe { sys::Mpv::from_library(library) }.map_err(|source| LoadError::LoadFailed {
        path: path.to_path_buf(),
        source,
    })?;
    let client_api = (reported >> 16, reported & 0xffff);
    tracing::info!(path = %path.display(), major = client_api.0, minor = client_api.1, "loaded libmpv");
    Ok(Mpv {
        api,
        path: path.to_path_buf(),
        client_api,
    })
}

/// `c_ulong` is 32 bits on Windows and 64 bits elsewhere; the API version always fits in 32.
#[allow(clippy::useless_conversion, clippy::unnecessary_cast)]
fn version_u32(raw: c_ulong) -> u32 {
    u32::try_from(u64::from(raw)).unwrap_or(u32::MAX)
}

#[cfg(windows)]
fn open(path: &Path) -> Result<libloading::Library, LoadError> {
    use libloading::os::windows::{LOAD_WITH_ALTERED_SEARCH_PATH, Library};

    // SAFETY: loading a library runs its DllMain, which is the point of loading libmpv. The path is absolute and
    // was resolved by this crate (never a bare name), and ALTERED_SEARCH_PATH makes the DLL's own dependencies
    // resolve from its folder instead of the current directory.
    unsafe { Library::load_with_flags(path, LOAD_WITH_ALTERED_SEARCH_PATH) }
        .map(libloading::Library::from)
        .map_err(|source| {
            // ERROR_BAD_EXE_FORMAT: a 64-bit DLL in a 32-bit process or the reverse, or not a PE file at all.
            // libloading keeps the code private; its Debug output is the bare number.
            let bad_exe_format =
                matches!(&source, libloading::Error::LoadLibraryExW { source: code } if format!("{code:?}") == "193");
            if bad_exe_format {
                LoadError::WrongArchitecture {
                    path: path.to_path_buf(),
                }
            } else {
                LoadError::LoadFailed {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })
}

#[cfg(not(windows))]
fn open(path: &Path) -> Result<libloading::Library, LoadError> {
    // SAFETY: loading a library runs its initializers, which is the point of loading libmpv. The path is
    // absolute and was resolved by this crate.
    unsafe { libloading::Library::new(path) }.map_err(|source| LoadError::LoadFailed {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;

    use super::*;

    /// A scratch folder removed on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("sucrose-engine-mpv-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create scratch folder");
            Self(path)
        }

        fn folder(&self, name: &str) -> PathBuf {
            let folder = self.0.join(name);
            fs::create_dir_all(&folder).expect("create folder");
            folder
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn path_var(entries: &[PathBuf]) -> OsString {
        std::env::join_paths(entries).expect("join PATH entries")
    }

    fn files_in(folders: &[&PathBuf]) -> Vec<PathBuf> {
        folders
            .iter()
            .flat_map(|f| LIBRARY_NAMES.iter().map(move |n| f.join(n)))
            .collect()
    }

    #[test]
    fn candidates_search_the_exe_folder_first_then_absolute_path_entries() {
        let scratch = Scratch::new("order");
        let (exe, a, b) = (scratch.folder("exe"), scratch.folder("a"), scratch.folder("b"));
        let found = candidates(&exe, Some(&path_var(&[a.clone(), b.clone()])));
        assert_eq!(found, files_in(&[&exe, &a, &b]));
    }

    #[test]
    fn candidates_skip_relative_empty_and_duplicate_path_entries() {
        let scratch = Scratch::new("skip");
        let (exe, a) = (scratch.folder("exe"), scratch.folder("a"));
        let entries = [
            PathBuf::from("."),
            PathBuf::from("bin"),
            PathBuf::new(),
            a.clone(),
            a.clone(),
            exe.clone(),
        ];
        assert_eq!(candidates(&exe, Some(&path_var(&entries))), files_in(&[&exe, &a]));
    }

    #[test]
    fn candidates_without_path_only_search_the_exe_folder() {
        let scratch = Scratch::new("no-path");
        let exe = scratch.folder("exe");
        assert_eq!(candidates(&exe, None), files_in(&[&exe]));
    }

    #[test]
    fn locate_returns_none_when_no_candidate_exists() {
        let scratch = Scratch::new("none");
        let (exe, a) = (scratch.folder("exe"), scratch.folder("a"));
        assert_eq!(locate(&exe, Some(&path_var(&[a]))), None);
    }

    #[test]
    fn locate_prefers_the_exe_folder_over_path() {
        let scratch = Scratch::new("prefer");
        let (exe, a) = (scratch.folder("exe"), scratch.folder("a"));
        fs::write(exe.join(LIBRARY_NAMES[0]), b"exe copy").expect("write");
        fs::write(a.join(LIBRARY_NAMES[0]), b"path copy").expect("write");
        assert_eq!(locate(&exe, Some(&path_var(&[a]))), Some(exe.join(LIBRARY_NAMES[0])));
    }

    #[test]
    fn locate_finds_a_library_on_an_absolute_path_entry() {
        let scratch = Scratch::new("on-path");
        let (exe, a) = (scratch.folder("exe"), scratch.folder("a"));
        fs::write(a.join(LIBRARY_NAMES[0]), b"path copy").expect("write");
        assert_eq!(
            locate(&exe, Some(&path_var(std::slice::from_ref(&a)))),
            Some(a.join(LIBRARY_NAMES[0]))
        );
    }

    #[test]
    fn load_rejects_relative_paths() {
        let error = load(Path::new("libmpv-2.dll")).expect_err("a relative path must be refused");
        assert!(matches!(error, LoadError::NotAbsolute { .. }), "{error:?}");
    }

    #[test]
    fn load_reports_a_file_that_is_not_a_library() {
        let scratch = Scratch::new("not-a-library");
        let fake = scratch.0.join(LIBRARY_NAMES[0]);
        fs::write(&fake, b"this is text, not a library").expect("write");
        let error = load(&fake).expect_err("a text file is not a library");
        assert!(
            matches!(
                error,
                LoadError::LoadFailed { .. } | LoadError::WrongArchitecture { .. }
            ),
            "{error:?}"
        );
        assert!(error.to_string().contains(&fake.display().to_string()), "{error}");
    }

    #[test]
    fn resolve_with_nothing_found_lists_the_searched_paths_and_hints_at_the_fix() {
        let scratch = Scratch::new("not-found");
        let (exe, a) = (scratch.folder("exe"), scratch.folder("a"));
        let var = path_var(&[a]);
        let error = resolve(Some(&exe), Some(&var)).expect_err("nothing exists");
        let LoadError::NotFound { searched } = &error else {
            panic!("expected NotFound, got {error:?}");
        };
        assert_eq!(searched, &candidates(&exe, Some(&var)));
        assert!(error.to_string().contains(&exe.display().to_string()), "{error}");
        assert!(error.hint().contains("cargo xtask fetch-engines"));
        assert!(error.hint().contains("reinstall Sucrose"));
    }

    #[test]
    fn every_error_has_a_hint() {
        let path = PathBuf::from("x");
        let errors = [
            LoadError::NotFound { searched: Vec::new() },
            LoadError::NotAbsolute { path: path.clone() },
            LoadError::WrongArchitecture { path },
            LoadError::ApiTooOld {
                found: 0,
                required: 2 << 16,
            },
        ];
        for error in errors {
            assert!(!error.hint().is_empty(), "{error:?}");
            assert!(!error.to_string().is_empty());
        }
    }
}
