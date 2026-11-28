//! The service's property host: the only writer of property files (Pattern 6, PROP-02, D-20).
//!
//! The Portal's editor asks to open a theme's properties and sends one change at a time; the host keeps the document in
//! memory, checks every change against it and saves it as the Rust-owned working copy
//! (`<data_dir>/Properties/<Theme><Type>`). The theme folder's own file and the C# app's
//! `Cache\Properties` are never written (D-20). The host does no process or socket work, so every rule
//! here is tested without a renderer: the service only passes the clock and routes the answers.

use std::{
    collections::HashMap,
    fs,
    io::{self, Write as _},
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};

use sucrose_core::properties::{
    Control, ControlKind, PROPERTIES_FILE, PropertiesError, SucroseProperties, parse_properties, properties_text,
    read_properties,
};
use sucrose_ipc::{blob::BLOB_MAX, portal::MAX_PROPERTY_CONTROL};

/// The most often a working copy is written while the user keeps changing it (a drag sends a change every 50 ms).
pub const WRITE_INTERVAL: Duration = Duration::from_millis(300);

/// The kind of a web theme's own property file: the working copy is `<Theme>Base`.
pub const BASE: &str = "Base";

/// The name of a working copy: the theme folder and the kind, as the C# app names its caches.
#[must_use]
pub fn working_copy_name(folder: &str, kind: &str) -> String {
    format!("{folder}{kind}")
}

/// Whether `folder` is one plain name that cannot leave the Properties folder: no separator, no drive
/// prefix, no `.`/`..`, not empty.
fn plain_name(folder: &str) -> bool {
    let mut parts = Path::new(folder).components();
    matches!((parts.next(), parts.next()), (Some(Component::Normal(_)), None))
        && !folder.contains(['/', '\\', ':'])
        && folder.len() <= sucrose_ipc::portal::MAX_PROPERTY_NAME
}

/// Whether a file of the C# cache is a delta (`<guid>.<Theme><Type>`: the per-library overrides), which D-20 does not
/// import.
#[must_use]
pub fn is_delta(name: &str) -> bool {
    let Some((head, _)) = name.split_once('.') else {
        return false;
    };
    head.len() == 36
        && head.char_indices().all(|(index, c)| match index {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// D-20: copies every file of the C# cache `from` that is not a delta into `to`, with the same name, once. Nothing
/// happens when `to` exists already (the import ran, or the user has working copies) or when `from` does not exist.
/// `from` is only read. The files are copied into a staging folder that is renamed to `to` at the end, so a
/// crash in the middle leaves no half import behind. Returns how many files were copied.
///
/// # Errors
/// The I/O error that stopped the copy; the staging folder is removed then.
pub fn ensure_imported(from: Option<&Path>, to: &Path) -> io::Result<usize> {
    let Some(from) = from.filter(|from| from.is_dir()) else {
        return Ok(0);
    };
    if to.exists() {
        return Ok(0);
    }
    let mut staging = to.as_os_str().to_owned();
    staging.push(".importing");
    let staging = PathBuf::from(staging);
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir_all(&staging)?;
    let copy = || -> io::Result<usize> {
        let mut copied = 0;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(text) = name.to_str() else {
                continue;
            };
            if is_delta(text) || !entry.file_type()?.is_file() {
                continue;
            }
            fs::copy(entry.path(), staging.join(&name))?;
            copied += 1;
        }
        fs::rename(&staging, to)?;
        Ok(copied)
    };
    copy().inspect_err(|_| {
        let _ = fs::remove_dir_all(&staging);
    })
}

/// Writes `text` to `path` through a temporary file in the same folder: written, synced, then renamed over the target, so
/// a crash leaves the old file or the new one, never half of one.
fn write_atomic(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    let written = (|| -> io::Result<()> {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written
}

/// Why a theme's properties could not be opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenError {
    /// The theme has no properties file, or its list is empty.
    NoProperties,
    /// The file or the working copy cannot be read, or the document is bigger than a blob.
    Unreadable,
}

/// A document the editor can show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    /// The working copy as its JSON text.
    pub bytes: Vec<u8>,
    /// The id of the blob that carries it.
    pub blob_id: u32,
}

/// Why a change was refused; nothing changes then.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The theme's properties are not open.
    NotOpen,
    /// A request text is too long, or the folder is not a plain name.
    TooLong,
    /// The control is not a JSON object of a control.
    Malformed,
    /// The working copy has no such key.
    UnknownKey,
    /// The control has another type than the one in the working copy.
    WrongType,
    /// A label, a button or a type this build does not know has no value to change.
    NotEditable,
    /// The value cannot be a value of this control (a dropdown index outside its list).
    OutOfRange,
}

struct Open {
    copy: PathBuf,
    properties: SucroseProperties,
    /// The instant the next write is due; set by the first change after a write and not moved by later ones, so a
    /// drag writes every [`WRITE_INTERVAL`].
    due: Option<Instant>,
    /// Whether the document has changes the working copy does not hold yet.
    dirty: bool,
}

/// The property documents the Portal has open, and the working copies they save to.
pub struct PropertiesHost {
    /// `<data_dir>/Properties`, or `None` when the OS reports no data folder.
    dir: Option<PathBuf>,
    open: HashMap<String, Open>,
    next_blob: u32,
}

impl PropertiesHost {
    /// A host that saves into `dir`.
    #[must_use]
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self {
            dir,
            open: HashMap::new(),
            next_blob: 0,
        }
    }

    /// Opens the properties of the theme in library folder `folder`, whose own file is `theme_dir/SucroseProperties.json`.
    /// The working copy is read when there is one (the user's values), the theme's file otherwise; opening writes
    /// nothing. A document that is open already is returned as it is in memory.
    ///
    /// # Errors
    /// [`OpenError`] when there is nothing to edit or it cannot be read.
    pub fn open(&mut self, folder: &str, theme_dir: &Path) -> Result<Opened, OpenError> {
        if !plain_name(folder) {
            return Err(OpenError::Unreadable);
        }
        let dir = self.dir.clone().ok_or(OpenError::Unreadable)?;
        if !self.open.contains_key(folder) {
            let copy = dir.join(working_copy_name(folder, BASE));
            let properties = match read_properties(&copy) {
                Ok(properties) => properties,
                Err(PropertiesError::Missing(_)) => {
                    read_properties(&theme_dir.join(PROPERTIES_FILE)).map_err(|error| match error {
                        PropertiesError::Missing(_) => OpenError::NoProperties,
                        _ => OpenError::Unreadable,
                    })?
                }
                // An unreadable copy is never replaced by the theme's file: it may be the user's, and the next save
                // would overwrite it.
                Err(_) => return Err(OpenError::Unreadable),
            };
            if properties.list.is_empty() {
                return Err(OpenError::NoProperties);
            }
            self.open.insert(
                folder.to_owned(),
                Open {
                    copy,
                    properties,
                    due: None,
                    dirty: false,
                },
            );
        }
        let document = self.open.get(folder).ok_or(OpenError::Unreadable)?;
        let bytes = properties_text(&document.properties)
            .map_err(|_| OpenError::Unreadable)?
            .into_bytes();
        if bytes.len() > BLOB_MAX {
            self.open.remove(folder);
            return Err(OpenError::Unreadable);
        }
        self.next_blob = self.next_blob.wrapping_add(1);
        Ok(Opened {
            bytes,
            blob_id: self.next_blob,
        })
    }

    /// Whether the properties of `folder` are open.
    #[cfg(test)]
    fn is_open(&self, folder: &str) -> bool {
        self.open.contains_key(folder)
    }

    /// Applies one change of the editor: `control_json` is the control object as the editor holds it. Only its value is
    /// taken: the type, the bounds, the texts and the items stay what the working copy says, and a number is held
    /// inside its bounds. Returns the control as it is now (the legacy JSON object text), which is what the renderers
    /// get. The write is due `WRITE_INTERVAL` after the first change that follows a write.
    ///
    /// # Errors
    /// [`Refusal`]; the document is untouched then.
    pub fn change(&mut self, folder: &str, key: &str, control_json: &str, now: Instant) -> Result<String, Refusal> {
        if control_json.len() > MAX_PROPERTY_CONTROL
            || key.len() > sucrose_ipc::portal::MAX_PROPERTY_NAME
            || !plain_name(folder)
        {
            return Err(Refusal::TooLong);
        }
        let document = self.open.get_mut(folder).ok_or(Refusal::NotOpen)?;
        let incoming = parse_control(key, control_json)?;
        let position = document
            .properties
            .list
            .iter()
            .position(|(name, _)| name == key)
            .ok_or(Refusal::UnknownKey)?;
        let stored = &document.properties.list[position].1;
        let kind = merged(&stored.kind, &incoming.kind)?;
        let applied = Control { kind, ..stored.clone() };
        let text = serde_json::to_string(&applied.to_json_value()).map_err(|_| Refusal::Malformed)?;
        document.properties.list[position].1 = applied;
        document.dirty = true;
        document.due.get_or_insert(now + WRITE_INTERVAL);
        Ok(text)
    }

    /// The instant the earliest due write falls, if a write is waiting.
    #[must_use]
    pub fn next_due(&self) -> Option<Instant> {
        self.open.values().filter_map(|document| document.due).min()
    }

    /// Writes the documents whose write is due at `now`. Returns each folder that was tried with its result; a document
    /// that failed stays dirty and is tried again when it closes.
    pub fn flush_due(&mut self, now: Instant) -> Vec<(String, io::Result<()>)> {
        let due: Vec<String> = self
            .open
            .iter()
            .filter(|(_, document)| document.due.is_some_and(|due| due <= now))
            .map(|(folder, _)| folder.clone())
            .collect();
        due.into_iter()
            .map(|folder| {
                let result = self.write(&folder);
                (folder, result)
            })
            .collect()
    }

    /// Writes `folder`'s document now when it has changes, and forgets it: the editor closed.
    pub fn close(&mut self, folder: &str) -> Option<io::Result<()>> {
        let result = self.open.contains_key(folder).then(|| self.write(folder));
        self.open.remove(folder);
        result
    }

    /// Closes every document: the Portal left or the service stops.
    pub fn close_all(&mut self) -> Vec<(String, io::Result<()>)> {
        let folders: Vec<String> = self.open.keys().cloned().collect();
        folders
            .into_iter()
            .filter_map(|folder| self.close(&folder).map(|result| (folder, result)))
            .collect()
    }

    /// Writes the working copy of `folder` when it is dirty.
    fn write(&mut self, folder: &str) -> io::Result<()> {
        let Some(document) = self.open.get_mut(folder) else {
            return Ok(());
        };
        document.due = None;
        if !document.dirty {
            return Ok(());
        }
        let text = properties_text(&document.properties)?;
        write_atomic(&document.copy, &text)?;
        document.dirty = false;
        Ok(())
    }
}

/// Reads the control of a change as a one-entry property list, so it takes the same rules as a file.
fn parse_control(key: &str, control_json: &str) -> Result<Control, Refusal> {
    let value: serde_json::Value = serde_json::from_str(control_json).map_err(|_| Refusal::Malformed)?;
    if !value.is_object() {
        return Err(Refusal::Malformed);
    }
    let document = serde_json::json!({ "PropertyList": { key: value } });
    let mut properties = parse_properties(&document.to_string()).map_err(|_| Refusal::Malformed)?;
    properties
        .list
        .pop()
        .map(|(_, control)| control)
        .ok_or(Refusal::Malformed)
}

/// A number held inside its bounds; bounds in the wrong order leave it alone.
fn held(value: f64, min: f64, max: f64) -> f64 {
    if min <= max { value.clamp(min, max) } else { value }
}

/// The stored kind with the value of `incoming`: nothing but the value comes from the editor.
fn merged(stored: &ControlKind, incoming: &ControlKind) -> Result<ControlKind, Refusal> {
    match (stored, incoming) {
        (ControlKind::Slider { min, max, step, .. }, ControlKind::Slider { value, .. }) => Ok(ControlKind::Slider {
            min: *min,
            max: *max,
            step: *step,
            value: held(*value, *min, *max),
        }),
        (ControlKind::Numberbox { min, max, places, .. }, ControlKind::Numberbox { value, .. }) => {
            Ok(ControlKind::Numberbox {
                min: *min,
                max: *max,
                places: *places,
                value: held(*value, *min, *max),
            })
        }
        (ControlKind::Textbox { .. }, ControlKind::Textbox { value }) => {
            Ok(ControlKind::Textbox { value: value.clone() })
        }
        (ControlKind::Passwordbox { .. }, ControlKind::Passwordbox { value }) => {
            Ok(ControlKind::Passwordbox { value: value.clone() })
        }
        (ControlKind::Colorpicker { .. }, ControlKind::Colorpicker { value }) => {
            Ok(ControlKind::Colorpicker { value: value.clone() })
        }
        (ControlKind::Checkbox { .. }, ControlKind::Checkbox { value }) => Ok(ControlKind::Checkbox { value: *value }),
        (ControlKind::Dropdown { items, .. }, ControlKind::Dropdown { value, .. }) => {
            let in_list = usize::try_from(*value).is_ok_and(|index| index < items.len());
            if in_list {
                Ok(ControlKind::dropdown(*value, items.clone()))
            } else {
                Err(Refusal::OutOfRange)
            }
        }
        (
            ControlKind::Filedropdown {
                desc,
                title,
                folder,
                filter,
                files,
                ..
            },
            ControlKind::Filedropdown { value, .. },
        ) => Ok(ControlKind::Filedropdown {
            desc: desc.clone(),
            value: value.clone(),
            title: title.clone(),
            folder: folder.clone(),
            filter: filter.clone(),
            files: files.clone(),
        }),
        (ControlKind::Label { .. } | ControlKind::Button { .. } | ControlKind::Unknown { .. }, _) => {
            Err(Refusal::NotEditable)
        }
        _ => Err(Refusal::WrongType),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    /// A scratch folder under the temp folder, removed on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "sucrose-properties-{name}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("scratch folder");
            Self(dir)
        }

        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const THEME_FILE: &str = r#"{
  "PropertyListener": "L('{0}', {1});",
  "PropertyList": {
    "amount": { "type": "slider", "text": "Amount", "min": -5, "max": 5, "step": 0.5, "value": 0 },
    "name": { "type": "textbox", "text": "Name", "value": "Sucrose" },
    "mode": { "type": "dropdown", "text": "Mode", "value": 1, "items": ["Flat", "Vertical", "Horizontal"] },
    "title": { "type": "label", "text": "Title", "value": "T" }
  }
}"#;

    const GUID: &str = "0f8fad5b-d9cb-469f-a165-70867728950e";

    /// A host over a scratch data folder and a theme folder that has its own properties file.
    fn setup(name: &str) -> (Scratch, PropertiesHost, PathBuf) {
        let scratch = Scratch::new(name);
        let theme = scratch.path("Library").join("calls");
        fs::create_dir_all(&theme).expect("theme folder");
        fs::write(theme.join(PROPERTIES_FILE), THEME_FILE).expect("theme file");
        let host = PropertiesHost::new(Some(scratch.path("Properties")));
        (scratch, host, theme)
    }

    fn slider(value: f64) -> String {
        format!(r#"{{"type":"slider","text":"x","min":0,"max":100,"step":1,"value":{value}}}"#)
    }

    fn working_copy(scratch: &Scratch) -> String {
        fs::read_to_string(scratch.path("Properties").join("callsBase")).expect("working copy")
    }

    #[test]
    fn the_working_copy_is_named_after_the_theme_and_the_kind() {
        assert_eq!(working_copy_name("calls", BASE), "callsBase");
        assert_eq!(working_copy_name("My Theme", "MpvPlayer"), "My ThemeMpvPlayer");
    }

    #[test]
    fn a_delta_is_a_guid_a_dot_and_a_name() {
        assert!(is_delta(&format!("{GUID}.callsBase")));
        assert!(!is_delta("callsBase"));
        assert!(!is_delta("a.b"));
        assert!(
            !is_delta("0f8fad5b-d9cb-469f-a165-70867728950.callsBase"),
            "a short guid"
        );
        assert!(
            !is_delta("0f8fad5bxd9cb-469f-a165-70867728950e.callsBase"),
            "a dash is missing"
        );
    }

    #[test]
    fn the_import_copies_every_file_but_the_deltas_and_never_touches_the_source() {
        let scratch = Scratch::new("import");
        let from = scratch.path("legacy");
        fs::create_dir_all(&from).expect("legacy folder");
        fs::write(from.join("aBase"), b"base").expect("write");
        fs::write(from.join("bMpvPlayer"), b"mpv").expect("write");
        fs::write(from.join(format!("{GUID}.aBase")), b"delta").expect("write");
        let before: Vec<_> = fs::read_dir(&from)
            .expect("read")
            .map(|e| e.expect("entry").file_name())
            .collect();
        let to = scratch.path("Properties");

        let copied = ensure_imported(Some(&from), &to).expect("import");

        assert_eq!(copied, 2);
        assert_eq!(fs::read(to.join("aBase")).expect("copy"), b"base");
        assert_eq!(fs::read(to.join("bMpvPlayer")).expect("copy"), b"mpv");
        assert!(!to.join(format!("{GUID}.aBase")).exists());
        let after: Vec<_> = fs::read_dir(&from)
            .expect("read")
            .map(|e| e.expect("entry").file_name())
            .collect();
        assert_eq!(
            before.len(),
            after.len(),
            "nothing was written, renamed or removed in the source"
        );
        assert_eq!(fs::read(from.join("aBase")).expect("source"), b"base");
        assert!(
            !scratch.path("Properties.importing").exists(),
            "no staging folder is left"
        );
    }

    #[test]
    fn the_import_runs_once_and_leaves_an_existing_folder_alone() {
        let scratch = Scratch::new("once");
        let from = scratch.path("legacy");
        fs::create_dir_all(&from).expect("legacy folder");
        fs::write(from.join("aBase"), b"base").expect("write");
        let to = scratch.path("Properties");
        assert_eq!(ensure_imported(Some(&from), &to).expect("import"), 1);
        fs::write(from.join("cBase"), b"later").expect("write");
        fs::write(to.join("aBase"), b"edited").expect("edit the copy");

        assert_eq!(ensure_imported(Some(&from), &to).expect("second call"), 0);

        assert!(!to.join("cBase").exists());
        assert_eq!(fs::read(to.join("aBase")).expect("copy"), b"edited");
        // A folder that exists but is empty counts as existing: the user's own choice is never overwritten.
        let empty = scratch.path("Empty");
        fs::create_dir_all(&empty).expect("folder");
        assert_eq!(ensure_imported(Some(&from), &empty).expect("empty"), 0);
    }

    #[test]
    fn a_missing_legacy_folder_copies_nothing() {
        let scratch = Scratch::new("missing");
        let to = scratch.path("Properties");

        assert_eq!(ensure_imported(Some(&scratch.path("nope")), &to).expect("import"), 0);
        assert_eq!(ensure_imported(None, &to).expect("import"), 0);
        assert!(!to.exists());
    }

    #[test]
    fn opening_reads_the_theme_file_and_writes_nothing() {
        let (scratch, mut host, theme) = setup("open");

        let opened = host.open("calls", &theme).expect("open");

        let text = String::from_utf8(opened.bytes).expect("utf8");
        assert!(text.contains("\"amount\""));
        assert!(!scratch.path("Properties").exists(), "opening writes nothing");
        assert!(host.is_open("calls"));
    }

    #[test]
    fn opening_prefers_the_working_copy_and_refuses_an_unreadable_one() {
        let (scratch, mut host, theme) = setup("copy");
        fs::create_dir_all(scratch.path("Properties")).expect("folder");
        fs::write(
            scratch.path("Properties").join("callsBase"),
            THEME_FILE.replace("\"value\": 0 }", "\"value\": 3 }"),
        )
        .expect("copy");

        let opened = host.open("calls", &theme).expect("open");

        assert!(String::from_utf8(opened.bytes).expect("utf8").contains("\"value\": 3"));
        let (scratch, mut host, theme) = setup("broken");
        fs::create_dir_all(scratch.path("Properties")).expect("folder");
        fs::write(scratch.path("Properties").join("callsBase"), "{ not json").expect("copy");
        assert_eq!(host.open("calls", &theme), Err(OpenError::Unreadable));
        assert_eq!(
            fs::read_to_string(scratch.path("Properties").join("callsBase")).expect("copy"),
            "{ not json",
            "the unreadable copy stays as it was"
        );
    }

    #[test]
    fn a_theme_with_no_properties_or_an_unreadable_file_fails_with_its_reason() {
        let (_scratch, mut host, theme) = setup("none");
        fs::remove_file(theme.join(PROPERTIES_FILE)).expect("remove");
        assert_eq!(host.open("calls", &theme), Err(OpenError::NoProperties));
        fs::write(theme.join(PROPERTIES_FILE), "{ not json").expect("write");
        assert_eq!(host.open("calls", &theme), Err(OpenError::Unreadable));
        fs::write(theme.join(PROPERTIES_FILE), r#"{"PropertyList":{}}"#).expect("write");
        assert_eq!(host.open("calls", &theme), Err(OpenError::NoProperties));
        assert_eq!(
            host.open("../x", &theme),
            Err(OpenError::Unreadable),
            "not a plain name"
        );
    }

    #[test]
    fn a_change_is_held_inside_its_bounds_and_written_after_the_interval() {
        let (scratch, mut host, theme) = setup("change");
        host.open("calls", &theme).expect("open");
        let now = Instant::now();

        let applied = host.change("calls", "amount", &slider(2.5), now).expect("change");

        assert!(applied.contains("\"value\":2.5"), "{applied}");
        assert!(
            applied.contains("\"min\":-5.0"),
            "the bounds stay the working copy's: {applied}"
        );
        assert!(host.flush_due(now).is_empty(), "not due yet");
        assert!(!scratch.path("Properties").exists());
        let written = host.flush_due(now + WRITE_INTERVAL);
        assert_eq!(written.len(), 1);
        assert!(written[0].1.is_ok());
        assert!(working_copy(&scratch).contains("\"value\": 2.5"));
        let clamped = host.change("calls", "amount", &slider(900.0), now).expect("change");
        assert!(clamped.contains("\"value\":5.0"), "{clamped}");
        assert!(
            fs::read_to_string(theme.join(PROPERTIES_FILE)).expect("theme file") == THEME_FILE,
            "the theme's own file is never written"
        );
    }

    #[test]
    fn the_same_change_twice_leaves_the_same_bytes() {
        let (scratch, mut host, theme) = setup("twice");
        host.open("calls", &theme).expect("open");
        let now = Instant::now();
        let first = host.change("calls", "amount", &slider(2.5), now).expect("change");
        host.flush_due(now + WRITE_INTERVAL);
        let bytes = working_copy(&scratch);

        let second = host
            .change("calls", "amount", &slider(2.5), now + WRITE_INTERVAL)
            .expect("change");
        host.flush_due(now + WRITE_INTERVAL * 2);

        assert_eq!(first, second);
        assert_eq!(working_copy(&scratch), bytes);
    }

    #[test]
    fn a_change_of_an_unknown_key_or_another_type_is_refused_and_changes_nothing() {
        let (scratch, mut host, theme) = setup("refused");
        host.open("calls", &theme).expect("open");
        let now = Instant::now();

        assert_eq!(
            host.change("calls", "nope", &slider(1.0), now),
            Err(Refusal::UnknownKey)
        );
        assert_eq!(
            host.change("calls", "amount", r#"{"type":"checkbox","value":true}"#, now),
            Err(Refusal::WrongType)
        );
        assert_eq!(
            host.change("calls", "title", r#"{"type":"label","value":"x"}"#, now),
            Err(Refusal::NotEditable)
        );
        assert_eq!(
            host.change("calls", "mode", r#"{"type":"dropdown","value":7,"items":[]}"#, now),
            Err(Refusal::OutOfRange)
        );
        assert_eq!(host.change("calls", "amount", "[1]", now), Err(Refusal::Malformed));
        assert_eq!(host.change("calls", "amount", "nope", now), Err(Refusal::Malformed));
        assert_eq!(host.change("other", "amount", &slider(1.0), now), Err(Refusal::NotOpen));
        assert_eq!(
            host.change("calls", "amount", &" ".repeat(MAX_PROPERTY_CONTROL + 1), now),
            Err(Refusal::TooLong)
        );
        assert_eq!(host.next_due(), None, "no write is waiting");
        assert!(host.flush_due(now + WRITE_INTERVAL).is_empty());
        assert!(!scratch.path("Properties").exists());
    }

    #[test]
    fn only_the_value_comes_from_the_editor() {
        let (scratch, mut host, theme) = setup("value-only");
        host.open("calls", &theme).expect("open");
        let now = Instant::now();
        let hostile = r#"{"type":"slider","text":"Hacked","min":-1000,"max":1000,"step":1,"value":4,"x":1}"#;

        let applied = host.change("calls", "amount", hostile, now).expect("change");

        assert!(applied.contains("\"text\":\"Amount\""), "{applied}");
        assert!(applied.contains("\"max\":5.0"), "{applied}");
        assert!(applied.contains("\"value\":4.0"), "{applied}");
        host.close("calls");
        assert!(!working_copy(&scratch).contains("Hacked"));
    }

    #[test]
    fn a_fast_drag_ends_on_its_last_value_and_writes_at_most_every_interval() {
        let (scratch, mut host, theme) = setup("drag");
        host.open("calls", &theme).expect("open");
        let start = Instant::now();
        let mut writes = 0;

        for step in 0..30_u32 {
            let now = start + Duration::from_millis(u64::from(step) * 50);
            host.change("calls", "amount", &slider(f64::from(step)), now)
                .expect("change");
            writes += host.flush_due(now).len();
        }
        let end = start + Duration::from_millis(30 * 50);
        writes += host.flush_due(end + WRITE_INTERVAL).len();

        assert!(writes <= 6, "30 changes in 1.5 s wrote {writes} times");
        assert!(
            working_copy(&scratch).contains("\"value\": 5"),
            "the last value was clamped to 5"
        );
        let last = host.change("calls", "amount", &slider(3.0), end).expect("change");
        assert!(last.contains("\"value\":3.0"));
        host.flush_due(end + WRITE_INTERVAL);
        assert!(working_copy(&scratch).contains("\"value\": 3"));
    }

    #[test]
    fn closing_writes_a_waiting_change_at_once_and_forgets_the_document() {
        let (scratch, mut host, theme) = setup("close");
        host.open("calls", &theme).expect("open");
        host.change("calls", "name", r#"{"type":"textbox","value":"Rust"}"#, Instant::now())
            .expect("change");

        let result = host.close("calls").expect("it was open");

        assert!(result.is_ok());
        assert!(working_copy(&scratch).contains("\"value\": \"Rust\""));
        assert!(!host.is_open("calls"));
        assert!(host.close("calls").is_none());
        assert!(
            !scratch.path("Properties").join("callsBase.tmp").exists(),
            "no temporary file is left"
        );
    }

    #[test]
    fn a_working_copy_keeps_the_legacy_text_form() {
        let (scratch, mut host, theme) = setup("form");
        host.open("calls", &theme).expect("open");
        host.change("calls", "amount", &slider(1.0), Instant::now())
            .expect("change");
        host.close("calls");

        let text = working_copy(&scratch);

        assert!(text.contains("\r\n"), "CRLF line ends like the C# app");
        assert!(!text.ends_with('\n'));
        assert!(read_properties(&scratch.path("Properties").join("callsBase")).is_ok());
    }
}
