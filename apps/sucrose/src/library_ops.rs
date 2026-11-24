//! Library maintenance on a worker thread of the service: the showcase themes are copied into the library once and, when
//! the user wants it, corrupt themes go to the recycle bin (LIB-01, LIB-02, D-06, D-09, D-10). The service decides
//! everything from its own settings and its own scan; the Portal only asks.

use std::{
    collections::HashSet,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};

use sucrose_config::model::Settings;
use sucrose_core::{
    library::{CorruptReason, scan},
    paths,
    version::app_version,
};
use sucrose_ipc::portal::{MaintainReport, MoveRefusal};

/// What one maintenance run works on, copied from the settings so the worker never touches them.
#[derive(Debug, Clone)]
pub struct MaintainInput {
    /// The library folder; created when missing.
    pub location: PathBuf,
    /// The folder with the themes shipped for first use (`data_dir()/Showcase`), when the OS has a data folder.
    pub showcase: Option<PathBuf>,
    /// Folder names already recorded in `Warehouse.Showcase`.
    pub copied: Vec<String>,
    /// Whether corrupt themes go to the bin (`Library.DeleteCorrupt`).
    pub delete_corrupt: bool,
    /// The folder with the per-theme property caches (`Cache/Properties`).
    pub properties: Option<PathBuf>,
}

/// Where a corrupt theme goes: the OS recycle bin or trash. A trait so tests record the calls instead of touching a bin.
pub trait Trash {
    /// Moves `path` (a folder) to the bin. The error is a sentence for the log, never a path.
    fn move_to_trash(&self, path: &Path) -> Result<(), String>;
}

impl MaintainInput {
    /// What the settings say about the library, with the showcase folder of this data folder.
    pub fn from_settings(settings: &Settings, legacy_root: &Path) -> Self {
        Self {
            location: settings.library_location(legacy_root),
            showcase: paths::showcase_dir(),
            copied: settings.warehouse.showcase.clone(),
            delete_corrupt: settings.library.delete_corrupt,
            properties: paths::properties_cache_dir(),
        }
    }
}

/// How much one [`copy_tree`] copied.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CopyStats {
    /// Files copied.
    pub files: u64,
    /// Bytes copied.
    pub bytes: u64,
}

/// Copies the folder `from` to `to` (created), nested folders and files included; symbolic links are skipped, never
/// followed.
pub fn copy_tree(from: &Path, to: &Path) -> io::Result<CopyStats> {
    copy_tree_with(from, to, &|source, destination| fs::copy(source, destination))
}

/// How one file is copied; tests swap it for one that fails.
type CopyFile<'a> = &'a dyn Fn(&Path, &Path) -> io::Result<u64>;

/// [`copy_tree`] with the file copy given.
fn copy_tree_with(from: &Path, to: &Path, copy_file: CopyFile<'_>) -> io::Result<CopyStats> {
    fs::create_dir_all(to)?;
    let mut stats = CopyStats::default();
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        // The type of the entry itself: a link is neither a file nor a folder here, so it is never followed.
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            let inner = copy_tree_with(&entry.path(), &target, copy_file)?;
            stats.files += inner.files;
            stats.bytes += inner.bytes;
        } else if kind.is_file() {
            stats.bytes += copy_file(&entry.path(), &target)?;
            stats.files += 1;
        }
    }
    Ok(stats)
}

/// What a failed library move reports; the move was rolled back as far as it could be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveFailure {
    /// One sentence for the user and the log; it never holds a path.
    pub reason: String,
}

/// What a move did, for the log: entries are the top-level items of the library folder.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MoveStats {
    /// Top-level entries that arrived in the destination.
    pub entries: u32,
    /// Of those, how many were renamed (the rest were copied).
    pub renamed: u32,
    /// Files copied.
    pub files: u64,
    /// Bytes copied.
    pub bytes: u64,
}

/// A move whose content is in the destination and verified; the source still holds the copied entries. The caller writes
/// the setting and then calls [`finish_move`], or calls [`rollback_move`] when the write failed.
#[derive(Debug)]
pub struct MovePlanDone {
    source: PathBuf,
    destination: PathBuf,
    renamed: Vec<OsString>,
    copied: Vec<OsString>,
    stats: MoveStats,
}

impl MovePlanDone {
    /// What was moved.
    pub fn stats(&self) -> MoveStats {
        self.stats
    }
}

/// What [`finish_move`] removed from the old location.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FinishStats {
    /// Copied entries removed from the old folder.
    pub removed: u32,
    /// Copied entries that could not be removed and stay in the old folder.
    pub failed: u32,
}

/// The file operations of a move; tests swap them for ones that fail on demand.
pub trait MoveOps {
    /// Renames `from` to `to` (a file or a folder).
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Copies a file or a folder tree; links are skipped.
    fn copy_tree(&self, from: &Path, to: &Path) -> io::Result<CopyStats>;
    /// Removes a file or a folder tree.
    fn remove_entry(&self, path: &Path) -> io::Result<()>;
}

/// The operations on the real file system.
#[derive(Debug, Clone, Copy)]
pub struct RealOps;

impl MoveOps for RealOps {
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn copy_tree(&self, from: &Path, to: &Path) -> io::Result<CopyStats> {
        if fs::symlink_metadata(from)?.is_file() {
            fs::copy(from, to).map(|bytes| CopyStats { files: 1, bytes })
        } else {
            copy_tree(from, to)
        }
    }

    fn remove_entry(&self, path: &Path) -> io::Result<()> {
        if fs::symlink_metadata(path)?.is_dir() {
            fs::remove_dir_all(path)
        } else {
            remove_file_forced(path)
        }
    }
}

/// Removes a file, clearing the read-only flag first when that is what stops the removal (Windows).
fn remove_file_forced(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            let mut permissions = fs::metadata(path)?.permissions();
            #[allow(
                clippy::permissions_set_readonly_false,
                reason = "the file is the library's own and is being deleted right after"
            )]
            permissions.set_readonly(false);
            fs::set_permissions(path, permissions)?;
            fs::remove_file(path)
        }
        other => other,
    }
}

/// Whether `destination` can take the library of `source`, for a move that carries the content along: the folder
/// exists, is not the library, not inside it nor around it, is empty and can be written to.
pub fn check_destination(source: &Path, destination: &Path) -> Result<(), MoveRefusal> {
    check_with(source, destination, true, &write_probe)
}

/// The answer of [`check_destination`] for a change that leaves the themes where they are: the folder exists, is not the
/// library and can be written to; it may hold themes already.
pub fn check_location(source: &Path, destination: &Path) -> Result<(), MoveRefusal> {
    check_with(source, destination, false, &write_probe)
}

/// How a folder is probed for write access; tests swap it.
type Probe<'a> = &'a dyn Fn(&Path) -> io::Result<()>;

/// Creates and removes a file in `folder`: the only reliable test for write access on every OS.
fn write_probe(folder: &Path) -> io::Result<()> {
    let probe = folder.join(format!(".sucrose-write-test-{}", std::process::id()));
    fs::write(&probe, b"")?;
    fs::remove_file(&probe)
}

/// The checks of [`check_destination`] (`carry`) and [`check_location`] with the write probe given.
fn check_with(source: &Path, destination: &Path, carry: bool, probe: Probe<'_>) -> Result<(), MoveRefusal> {
    if !destination.is_dir() {
        return Err(MoveRefusal::Missing);
    }
    // Resolved, so a link, a `..` or another spelling of the same folder cannot get past the nesting check (T-03-52).
    let library = resolved(source);
    let target = resolved(destination);
    if library == target {
        return Err(MoveRefusal::Same);
    }
    if carry {
        if target.starts_with(&library) || library.starts_with(&target) {
            return Err(MoveRefusal::Nested);
        }
        let empty = fs::read_dir(destination).is_ok_and(|mut entries| entries.next().is_none());
        if !empty {
            return Err(MoveRefusal::NotEmpty);
        }
    }
    probe(destination).map_err(|_| MoveRefusal::NotWritable)
}

/// `path` with links and `..` resolved; the path itself when it does not exist (a library folder that was never created).
fn resolved(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Moves every top-level entry of `source` into `destination`: renamed when that works, else copied and verified by file
/// count and size. Nothing in `source` is deleted here. A failure puts everything back before it returns.
pub fn move_library(source: &Path, destination: &Path, ops: &dyn MoveOps) -> Result<MovePlanDone, MoveFailure> {
    let entries = entries_of(source).map_err(|error| failure("the library folder cannot be read", &error))?;
    let mut done = MovePlanDone {
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
        renamed: Vec::new(),
        copied: Vec::new(),
        stats: MoveStats::default(),
    };
    for entry in entries {
        let from = source.join(&entry.name);
        let to = destination.join(&entry.name);
        // Any rename error means "copy instead": a cross-volume move is the expected one (A2), and a file that is held
        // open is as well served by a copy.
        if ops.rename(&from, &to).is_ok() {
            done.renamed.push(entry.name);
            done.stats.renamed += 1;
            done.stats.entries += 1;
            continue;
        }
        // A link is never followed, so it cannot be copied: it stays where it is.
        if entry.is_link {
            tracing::debug!("a link in the library stays in the old folder");
            continue;
        }
        match copy_verified(ops, &from, &to) {
            Ok(copied) => {
                done.copied.push(entry.name);
                done.stats.entries += 1;
                done.stats.files += copied.files;
                done.stats.bytes += copied.bytes;
            }
            Err(error) => {
                // Whatever the failed copy left in the destination was created by it: the destination was empty.
                if to.symlink_metadata().is_ok()
                    && let Err(error) = ops.remove_entry(&to)
                {
                    tracing::warn!(%error, "what the failed copy created could not be removed");
                }
                let restored = restore(&done, ops);
                let reason = if restored {
                    error
                } else {
                    format!("{error}; some of the content could not be put back")
                };
                return Err(MoveFailure { reason });
            }
        }
    }
    Ok(done)
}

/// A top-level entry of the library folder.
struct Entry {
    name: OsString,
    is_link: bool,
}

/// The files, folders and links directly inside `folder`, by name.
fn entries_of(folder: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() || kind.is_file() || kind.is_symlink() {
            entries.push(Entry {
                name: entry.file_name(),
                is_link: kind.is_symlink(),
            });
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// A sentence for a failure: what went wrong and the OS's own words, never a path.
fn failure(what: &str, error: &io::Error) -> MoveFailure {
    MoveFailure {
        reason: format!("{what}: {error}"),
    }
}

/// Copies `from` to `to` and compares file count and total size of both sides.
fn copy_verified(ops: &dyn MoveOps, from: &Path, to: &Path) -> Result<CopyStats, String> {
    let copied = ops
        .copy_tree(from, to)
        .map_err(|error| format!("a file could not be copied to the new folder: {error}"))?;
    let before = measure(from).map_err(|error| format!("the copy could not be verified: {error}"))?;
    let after = measure(to).map_err(|error| format!("the copy could not be verified: {error}"))?;
    if before != after || copied != after {
        return Err("the copy in the new folder does not match the original".to_owned());
    }
    Ok(after)
}

/// File count and total size of `path` (a file or a folder tree); links are not counted, like the copy skips them.
fn measure(path: &Path) -> io::Result<CopyStats> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_file() {
        return Ok(CopyStats {
            files: 1,
            bytes: meta.len(),
        });
    }
    let mut stats = CopyStats::default();
    if meta.is_dir() {
        for entry in fs::read_dir(path)? {
            let inner = measure(&entry?.path())?;
            stats.files += inner.files;
            stats.bytes += inner.bytes;
        }
    }
    Ok(stats)
}

/// Puts back what `done` moved: renamed entries are renamed back, copies are removed. `true` when nothing is left over.
fn restore(done: &MovePlanDone, ops: &dyn MoveOps) -> bool {
    let mut all_back = true;
    for name in done.renamed.iter().rev() {
        if let Err(error) = ops.rename(&done.destination.join(name), &done.source.join(name)) {
            tracing::warn!(%error, "an entry could not be renamed back");
            all_back = false;
        }
    }
    for name in done.copied.iter().rev() {
        if let Err(error) = ops.remove_entry(&done.destination.join(name)) {
            tracing::warn!(%error, "a copied entry could not be removed from the new folder");
            all_back = false;
        }
    }
    all_back
}

/// Removes the copied entries from the old folder, then the old folder itself when it is empty. Renamed entries are
/// already gone from it.
pub fn finish_move(done: MovePlanDone, ops: &dyn MoveOps) -> FinishStats {
    let mut finished = FinishStats::default();
    for name in &done.copied {
        match ops.remove_entry(&done.source.join(name)) {
            Ok(()) => finished.removed += 1,
            Err(error) => {
                tracing::warn!(%error, "a copied entry could not be removed from the old folder");
                finished.failed += 1;
            }
        }
    }
    // Only an empty folder goes: a link or a file that could not be removed keeps its folder.
    if let Err(error) = fs::remove_dir(&done.source) {
        tracing::debug!(%error, "the old library folder stays");
    }
    finished
}

/// Puts a move back: renamed entries are renamed back and the copies in the destination are removed. `true` when
/// everything is back.
pub fn rollback_move(done: MovePlanDone, ops: &dyn MoveOps) -> bool {
    restore(&done, ops)
}

/// Runs the maintenance: the report and the showcase names to record in `Warehouse.Showcase`.
pub fn maintain(input: &MaintainInput, trash: &dyn Trash) -> (MaintainReport, Vec<String>) {
    maintain_with(input, trash, &|source, destination| fs::copy(source, destination))
}

/// [`maintain`] with the file copy given.
fn maintain_with(input: &MaintainInput, trash: &dyn Trash, copy_file: CopyFile<'_>) -> (MaintainReport, Vec<String>) {
    let mut report = MaintainReport {
        showcase_copied: 0,
        corrupt_moved: 0,
        corrupt_skipped: 0,
    };
    if let Err(error) = fs::create_dir_all(&input.location) {
        tracing::warn!(%error, "the library folder cannot be created, nothing is maintained");
        return (report, Vec::new());
    }
    let (copied, names) = copy_showcase(input, copy_file);
    report.showcase_copied = copied;
    if input.delete_corrupt {
        (report.corrupt_moved, report.corrupt_skipped) = trash_corrupt(input, trash);
    }
    (report, names)
}

/// Moves every folder the library scan classifies as corrupt to the bin and deletes that theme's property cache file
/// (legacy). The service scans here itself and trusts no list from the Portal (T-03-48). A folder that cannot be read, a
/// link, or one the bin refuses stays where it is. Nothing is ever deleted except a cache file. Returns how many folders
/// moved and how many stayed.
fn trash_corrupt(input: &MaintainInput, trash: &dyn Trash) -> (u32, u32) {
    let scan = match scan(&input.location, &app_version()) {
        Ok(scan) => scan,
        Err(error) => {
            tracing::warn!(%error, "the library cannot be scanned, no corrupt theme is moved");
            return (0, 0);
        }
    };
    let (mut moved, mut skipped) = (0, 0);
    for corrupt in scan.corrupt {
        // Unreadable is never a reason to move a folder; a link would be followed by the scan but is not ours to move.
        let is_link = corrupt
            .dir
            .symlink_metadata()
            .is_ok_and(|meta| meta.file_type().is_symlink());
        if corrupt.reason == CorruptReason::Unreadable || is_link {
            skipped += 1;
            continue;
        }
        match trash.move_to_trash(&corrupt.dir) {
            Ok(()) => {
                moved += 1;
                if let Some(properties) = &input.properties {
                    // The folder name is a directory entry name of the scan, so it cannot leave the cache folder.
                    let cache = properties.join(format!("{}.json", corrupt.folder));
                    if cache.is_file()
                        && let Err(error) = fs::remove_file(&cache)
                    {
                        tracing::debug!(%error, "the property cache of a moved theme could not be deleted");
                    }
                }
            }
            Err(error) => {
                tracing::warn!(%error, "a corrupt theme could not be moved to the bin, it stays");
                skipped += 1;
            }
        }
    }
    (moved, skipped)
}

/// Copies every folder of the showcase whose name is not recorded yet into the library. A name is returned only when the
/// folder is in the library afterwards: copied now, or already there (legacy: it was there before). An existing folder is
/// never written to. Returns the number of folders copied and the names to record.
fn copy_showcase(input: &MaintainInput, copy_file: CopyFile<'_>) -> (u32, Vec<String>) {
    let Some(showcase) = &input.showcase else {
        return (0, Vec::new());
    };
    // A showcase folder that is missing is the normal case after the first run, and under SUCROSE_DATA_DIR.
    let Ok(entries) = fs::read_dir(showcase) else {
        return (0, Vec::new());
    };
    let listed: HashSet<&str> = input.copied.iter().map(String::as_str).collect();
    let mut folders: Vec<_> = entries
        .filter_map(Result::ok)
        // A link is not followed: only real folders are showcase themes.
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .collect();
    folders.sort_by_key(fs::DirEntry::file_name);
    let (mut copied, mut names) = (0, Vec::new());
    for entry in folders {
        let name = entry.file_name().to_string_lossy().into_owned();
        if listed.contains(name.as_str()) {
            continue;
        }
        let destination = input.location.join(entry.file_name());
        if destination.symlink_metadata().is_ok() {
            names.push(name);
            continue;
        }
        match copy_tree_with(&entry.path(), &destination, copy_file) {
            Ok(stats) => {
                tracing::debug!(files = stats.files, bytes = stats.bytes, "a showcase theme was copied");
                copied += 1;
                names.push(name);
            }
            Err(error) => {
                tracing::warn!(%error, "a showcase theme could not be copied");
                // The destination did not exist before this copy, so everything in it was created by it.
                if let Err(error) = fs::remove_dir_all(&destination) {
                    tracing::warn!(%error, "what the failed copy created could not be removed");
                }
            }
        }
    }
    (copied, names)
}

/// Adds `names` to the recorded showcase list, each once.
pub fn record_names(list: &mut Vec<String>, names: &[String]) {
    for name in names {
        if !list.contains(name) {
            list.push(name.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        fs,
        path::{Path, PathBuf},
    };

    use super::*;

    /// A scratch tree of its own, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("sucrose-library-ops-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("the scratch folder can be created");
            Self(dir)
        }

        fn path(&self, relative: &str) -> PathBuf {
            self.0.join(relative)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// A showcase theme folder with an info file and a nested file.
    fn theme(root: &Path, name: &str) {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("assets")).expect("the theme folder can be created");
        fs::write(dir.join("SucroseInfo.json"), b"{}").expect("the info file can be written");
        fs::write(dir.join("assets").join("video.mp4"), b"0123456789").expect("the nested file can be written");
    }

    fn input(scratch: &Scratch, copied: &[&str]) -> MaintainInput {
        MaintainInput {
            location: scratch.path("Library"),
            showcase: Some(scratch.path("Showcase")),
            copied: copied.iter().map(|name| (*name).to_owned()).collect(),
            delete_corrupt: false,
            properties: Some(scratch.path("Properties")),
        }
    }

    /// A fake bin: records what it is given and fails when told to.
    #[derive(Default)]
    struct Recorder {
        moved: RefCell<Vec<PathBuf>>,
        fail: bool,
    }

    impl Trash for Recorder {
        fn move_to_trash(&self, path: &Path) -> Result<(), String> {
            if self.fail {
                return Err("the bin is full".to_owned());
            }
            self.moved.borrow_mut().push(path.to_path_buf());
            Ok(())
        }
    }

    impl Recorder {
        /// The folder names handed to the bin, in order.
        fn names(&self) -> Vec<String> {
            self.moved
                .borrow()
                .iter()
                .filter_map(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
                .collect()
        }
    }

    /// A folder of the library that is no theme: no `SucroseInfo.json`.
    fn corrupt_folder(library: &Path, name: &str) {
        fs::create_dir_all(library.join(name)).expect("the folder can be created");
        fs::write(library.join(name).join("readme.txt"), b"x").expect("the file can be written");
    }

    /// A theme folder the scan accepts, written for `app_version`.
    fn library_theme(library: &Path, name: &str, app_version: &str) {
        let dir = library.join(name);
        fs::create_dir_all(&dir).expect("the theme folder can be created");
        let info = format!(
            r#"{{"AppVersion":"{app_version}","Title":"T","Thumbnail":"th.jpg","Preview":"pv.gif","Type":3,"Source":"v.mp4"}}"#
        );
        fs::write(dir.join("SucroseInfo.json"), info).expect("the info file can be written");
        for file in ["th.jpg", "pv.gif", "v.mp4"] {
            fs::write(dir.join(file), b"x").expect("the theme file can be written");
        }
    }

    /// An input that moves corrupt themes, with a library of one valid theme, one that needs a newer app and two corrupt
    /// folders, and a property cache file for each of three folders.
    fn corrupt_scene(scratch: &Scratch) -> MaintainInput {
        let library = scratch.path("Library");
        library_theme(&library, "good", "26.10.19.0");
        library_theme(&library, "newer", "99.0.0.0");
        corrupt_folder(&library, "bad-one");
        corrupt_folder(&library, "bad-two");
        let properties = scratch.path("Properties");
        fs::create_dir_all(&properties).expect("the property cache can be created");
        for name in ["good", "bad-one", "bad-two"] {
            fs::write(properties.join(format!("{name}.json")), b"{}").expect("the cache file can be written");
        }
        MaintainInput {
            delete_corrupt: true,
            ..input(scratch, &[])
        }
    }

    #[test]
    fn corrupt_themes_stay_where_they_are_when_the_setting_is_off() {
        let scratch = Scratch::new("corrupt-off");
        let input = MaintainInput {
            delete_corrupt: false,
            ..corrupt_scene(&scratch)
        };
        let bin = Recorder::default();

        let (report, _) = maintain(&input, &bin);

        assert!(bin.names().is_empty());
        assert_eq!((report.corrupt_moved, report.corrupt_skipped), (0, 0));
        assert!(scratch.path("Properties/bad-one.json").is_file());
    }

    #[test]
    fn corrupt_folders_go_to_the_bin_once_and_their_property_file_is_deleted() {
        let scratch = Scratch::new("corrupt-on");
        let input = corrupt_scene(&scratch);
        let bin = Recorder::default();

        let (report, names) = maintain(&input, &bin);

        assert_eq!(
            bin.names(),
            ["bad-one", "bad-two"],
            "valid and newer themes are never moved"
        );
        assert_eq!((report.corrupt_moved, report.corrupt_skipped), (2, 0));
        assert!(names.is_empty());
        assert!(!scratch.path("Properties/bad-one.json").exists());
        assert!(!scratch.path("Properties/bad-two.json").exists());
        assert!(
            scratch.path("Properties/good.json").is_file(),
            "a theme that stays keeps its cache"
        );
    }

    #[test]
    fn corrupt_error_from_the_bin_counts_as_skipped_and_keeps_the_folder() {
        let scratch = Scratch::new("corrupt-fail");
        let input = corrupt_scene(&scratch);
        let bin = Recorder {
            fail: true,
            ..Recorder::default()
        };

        let (report, _) = maintain(&input, &bin);

        assert_eq!((report.corrupt_moved, report.corrupt_skipped), (0, 2));
        assert!(
            scratch.path("Library/bad-one").is_dir(),
            "the folder is never deleted by the corrupt step"
        );
        assert!(scratch.path("Library/bad-two").is_dir());
        assert!(
            scratch.path("Properties/bad-one.json").is_file(),
            "its cache stays with it"
        );
    }

    #[test]
    fn corrupt_step_leaves_a_library_of_valid_and_newer_themes_alone() {
        let scratch = Scratch::new("corrupt-none");
        let library = scratch.path("Library");
        library_theme(&library, "good", "26.10.19.0");
        library_theme(&library, "newer", "99.0.0.0");
        let input = MaintainInput {
            delete_corrupt: true,
            ..input(&scratch, &[])
        };
        let bin = Recorder::default();

        let (report, _) = maintain(&input, &bin);

        assert!(bin.names().is_empty());
        assert_eq!((report.corrupt_moved, report.corrupt_skipped), (0, 0));
    }

    #[test]
    fn copies_only_the_showcase_folders_that_are_not_listed() {
        let scratch = Scratch::new("unlisted");
        theme(&scratch.path("Showcase"), "A");
        theme(&scratch.path("Showcase"), "B");

        let (report, names) = maintain(&input(&scratch, &["A"]), &Recorder::default());

        assert_eq!(names, ["B"]);
        assert_eq!(report.showcase_copied, 1);
        assert!(scratch.path("Library/B/assets/video.mp4").is_file());
        assert!(!scratch.path("Library/A").exists());
    }

    #[test]
    fn with_every_name_listed_nothing_is_copied() {
        let scratch = Scratch::new("listed");
        theme(&scratch.path("Showcase"), "A");
        theme(&scratch.path("Showcase"), "B");

        let (report, names) = maintain(&input(&scratch, &["A", "B"]), &Recorder::default());

        assert!(names.is_empty());
        assert_eq!(report.showcase_copied, 0);
        assert!(scratch.path("Library").is_dir(), "the library folder is created anyway");
        assert_eq!(fs::read_dir(scratch.path("Library")).map(Iterator::count).ok(), Some(0));
    }

    #[test]
    fn an_existing_library_folder_is_not_overwritten_and_is_recorded() {
        let scratch = Scratch::new("existing");
        theme(&scratch.path("Showcase"), "B");
        fs::create_dir_all(scratch.path("Library/B")).expect("the library folder can be created");
        fs::write(scratch.path("Library/B/mine.txt"), b"mine").expect("the file can be written");

        let (report, names) = maintain(&input(&scratch, &[]), &Recorder::default());

        assert_eq!(names, ["B"], "the name is recorded: the folder was already there");
        assert_eq!(report.showcase_copied, 0, "nothing was copied");
        assert_eq!(
            fs::read(scratch.path("Library/B/mine.txt")).ok().as_deref(),
            Some(b"mine".as_slice())
        );
        assert!(!scratch.path("Library/B/SucroseInfo.json").exists());
    }

    #[test]
    fn a_missing_showcase_folder_copies_nothing() {
        let scratch = Scratch::new("no-showcase");

        let (report, names) = maintain(&input(&scratch, &[]), &Recorder::default());

        assert!(names.is_empty());
        assert_eq!(report.showcase_copied, 0);
        assert!(scratch.path("Library").is_dir());
    }

    #[test]
    fn copy_tree_copies_nested_folders_and_counts_files_and_bytes() {
        let scratch = Scratch::new("tree");
        theme(&scratch.0, "src");

        let stats = copy_tree(&scratch.path("src"), &scratch.path("dst")).expect("the copy succeeds");

        assert_eq!(stats, CopyStats { files: 2, bytes: 12 });
        assert!(scratch.path("dst/assets/video.mp4").is_file());
    }

    #[test]
    fn copy_tree_skips_symbolic_links() {
        let scratch = Scratch::new("links");
        theme(&scratch.0, "src");
        fs::write(scratch.path("outside.txt"), b"outside").expect("the file can be written");
        if !link(&scratch.path("outside.txt"), &scratch.path("src/link.txt")) {
            // Creating a link needs a privilege on some systems; the test has nothing to check without one.
            return;
        }

        let stats = copy_tree(&scratch.path("src"), &scratch.path("dst")).expect("the copy succeeds");

        assert!(!scratch.path("dst/link.txt").exists());
        assert_eq!(stats.files, 2);
    }

    /// Creates a file symbolic link; `false` when this system does not let the test do that.
    fn link(target: &Path, at: &Path) -> bool {
        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(target, at);
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_file(target, at);
        made.is_ok()
    }

    #[test]
    fn a_copy_that_fails_halfway_removes_what_it_created_and_records_nothing() {
        let scratch = Scratch::new("halfway");
        theme(&scratch.path("Showcase"), "B");
        // The info file copies, the nested video does not.
        let failing = |source: &Path, destination: &Path| {
            if source.file_name().is_some_and(|name| name == "video.mp4") {
                Err(io::Error::other("the disk is full"))
            } else {
                fs::copy(source, destination)
            }
        };

        let (report, names) = maintain_with(&input(&scratch, &[]), &Recorder::default(), &failing);

        assert!(names.is_empty(), "a failed copy is not recorded");
        assert_eq!(report.showcase_copied, 0);
        assert!(!scratch.path("Library/B").exists(), "the half copy is removed");
    }

    /// File operations that fail on demand and count what they are asked to do.
    #[derive(Default)]
    struct TestOps {
        /// Renames with a number in this range (0-based) fail.
        fail_rename: Option<std::ops::Range<usize>>,
        /// The copy with this number (0-based) fails after it created part of its destination.
        fail_copy_at: Option<usize>,
        renames: Cell<usize>,
        copies: Cell<usize>,
    }

    impl MoveOps for TestOps {
        fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
            let call = self.renames.get();
            self.renames.set(call + 1);
            if self.fail_rename.as_ref().is_some_and(|range| range.contains(&call)) {
                return Err(io::Error::from(io::ErrorKind::CrossesDevices));
            }
            RealOps.rename(from, to)
        }

        fn copy_tree(&self, from: &Path, to: &Path) -> io::Result<CopyStats> {
            let call = self.copies.get();
            self.copies.set(call + 1);
            if self.fail_copy_at == Some(call) {
                fs::create_dir_all(to)?;
                return Err(io::Error::other("the disk is full"));
            }
            RealOps.copy_tree(from, to)
        }

        fn remove_entry(&self, path: &Path) -> io::Result<()> {
            RealOps.remove_entry(path)
        }
    }

    /// A library of two themes, a loose file and a folder that is no theme; `Library` and an empty `New` next to it.
    fn move_scene(scratch: &Scratch) -> (PathBuf, PathBuf) {
        let library = scratch.path("Library");
        theme(&library, "Alpha");
        theme(&library, "Beta");
        corrupt_folder(&library, "Notes");
        fs::write(library.join("loose.txt"), b"loose").expect("the loose file can be written");
        let new = scratch.path("New");
        fs::create_dir_all(&new).expect("the new folder can be created");
        (library, new)
    }

    /// The names inside `folder`, sorted.
    fn names(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(folder)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    #[test]
    fn a_destination_with_anything_in_it_is_not_empty() {
        let scratch = Scratch::new("dest-not-empty");
        let (library, new) = move_scene(&scratch);
        fs::write(new.join("a.txt"), b"a").expect("the file can be written");

        assert_eq!(check_destination(&library, &new), Err(MoveRefusal::NotEmpty));
        // Without moving the themes along, a folder with themes in it is a fine new location.
        assert_eq!(check_location(&library, &new), Ok(()));
    }

    #[test]
    fn the_library_itself_is_the_same_folder_whatever_its_spelling() {
        let scratch = Scratch::new("dest-same");
        let (library, _) = move_scene(&scratch);
        let spelled = library.join("..").join("Library");

        assert_eq!(check_destination(&library, &library), Err(MoveRefusal::Same));
        assert_eq!(check_destination(&library, &spelled), Err(MoveRefusal::Same));
        assert_eq!(check_location(&library, &library), Err(MoveRefusal::Same));
    }

    #[test]
    fn a_folder_inside_the_library_or_around_it_is_nested() {
        let scratch = Scratch::new("dest-nested");
        let (library, _) = move_scene(&scratch);
        let inside = library.join("Inner");
        fs::create_dir_all(&inside).expect("the inner folder can be created");

        assert_eq!(check_destination(&library, &inside), Err(MoveRefusal::Nested));
        assert_eq!(check_destination(&inside, &library), Err(MoveRefusal::Nested));
        assert_eq!(check_destination(&library, &scratch.0), Err(MoveRefusal::Nested));
    }

    #[test]
    fn a_destination_that_is_not_a_folder_is_missing() {
        let scratch = Scratch::new("dest-missing");
        let (library, _) = move_scene(&scratch);
        fs::write(scratch.path("file.txt"), b"x").expect("the file can be written");

        assert_eq!(
            check_destination(&library, &scratch.path("nowhere")),
            Err(MoveRefusal::Missing)
        );
        assert_eq!(
            check_destination(&library, &scratch.path("file.txt")),
            Err(MoveRefusal::Missing)
        );
    }

    #[test]
    fn a_destination_where_the_probe_file_cannot_be_created_is_not_writable() {
        let scratch = Scratch::new("dest-readonly");
        let (library, new) = move_scene(&scratch);
        let denied = |_: &Path| Err(io::Error::from(io::ErrorKind::PermissionDenied));

        assert_eq!(check_with(&library, &new, true, &denied), Err(MoveRefusal::NotWritable));
        assert_eq!(
            check_with(&library, &new, false, &denied),
            Err(MoveRefusal::NotWritable)
        );
        assert_eq!(check_destination(&library, &new), Ok(()));
        assert!(names(&new).is_empty(), "the probe leaves nothing behind");
    }

    #[test]
    fn on_one_volume_every_entry_is_renamed_and_nothing_is_copied() {
        let scratch = Scratch::new("move-rename");
        let (library, new) = move_scene(&scratch);
        let ops = TestOps::default();

        let done = move_library(&library, &new, &ops).expect("the move succeeds");

        assert_eq!(ops.copies.get(), 0);
        assert_eq!(names(&new), ["Alpha", "Beta", "Notes", "loose.txt"]);
        assert!(names(&library).is_empty());
        assert_eq!(done.stats().entries, 4);
        assert_eq!(done.stats().renamed, 4);
        assert!(new.join("Alpha/assets/video.mp4").is_file());

        let finished = finish_move(done, &ops);

        assert_eq!(finished, FinishStats::default());
        assert!(!library.exists(), "the empty old folder is removed");
    }

    #[test]
    fn when_rename_fails_every_entry_is_copied_and_verified_and_the_source_waits_for_finish() {
        let scratch = Scratch::new("move-copy");
        let (library, new) = move_scene(&scratch);
        let ops = TestOps {
            fail_rename: Some(0..usize::MAX),
            ..TestOps::default()
        };

        let done = move_library(&library, &new, &ops).expect("the move succeeds");

        assert_eq!(ops.copies.get(), 4);
        assert_eq!(done.stats().renamed, 0);
        assert_eq!(done.stats().entries, 4);
        assert_eq!(
            done.stats().files,
            2 + 2 + 1 + 1,
            "two files per theme, one in Notes, the loose file"
        );
        assert_eq!(names(&new), ["Alpha", "Beta", "Notes", "loose.txt"]);
        assert_eq!(
            names(&library),
            ["Alpha", "Beta", "Notes", "loose.txt"],
            "the source is untouched"
        );

        let finished = finish_move(done, &ops);

        assert_eq!(finished, FinishStats { removed: 4, failed: 0 });
        assert!(!library.exists());
        assert_eq!(
            fs::read(new.join("loose.txt")).ok().as_deref(),
            Some(b"loose".as_slice())
        );
    }

    #[test]
    fn a_copy_that_fails_on_the_third_entry_removes_the_copies_and_leaves_the_source_whole() {
        let scratch = Scratch::new("move-copy-fails");
        let (library, new) = move_scene(&scratch);
        let ops = TestOps {
            fail_rename: Some(0..usize::MAX),
            fail_copy_at: Some(2),
            ..TestOps::default()
        };

        let failure = move_library(&library, &new, &ops).expect_err("the third copy fails");

        assert!(!failure.reason.is_empty());
        assert!(names(&new).is_empty(), "the destination is empty again");
        assert_eq!(names(&library), ["Alpha", "Beta", "Notes", "loose.txt"]);
        assert!(library.join("Beta/assets/video.mp4").is_file());
    }

    #[test]
    fn entries_renamed_before_a_failure_are_renamed_back() {
        let scratch = Scratch::new("move-mixed-fails");
        let (library, new) = move_scene(&scratch);
        // The first two renames work; the third fails, and so does the copy that replaces it.
        let ops = TestOps {
            fail_rename: Some(2..3),
            fail_copy_at: Some(0),
            ..TestOps::default()
        };

        let failure = move_library(&library, &new, &ops).expect_err("the copy of the third entry fails");

        assert!(!failure.reason.is_empty());
        assert!(names(&new).is_empty(), "the destination is empty again");
        assert_eq!(names(&library), ["Alpha", "Beta", "Notes", "loose.txt"]);
        assert!(library.join("Alpha/assets/video.mp4").is_file());
    }

    #[test]
    fn a_rollback_after_a_finished_copy_removes_the_copies() {
        let scratch = Scratch::new("move-rollback");
        let (library, new) = move_scene(&scratch);
        let ops = TestOps {
            fail_rename: Some(2..4),
            ..TestOps::default()
        };
        let done = move_library(&library, &new, &ops).expect("the move succeeds");
        assert_eq!(done.stats().renamed, 2);

        assert!(rollback_move(done, &ops), "everything is back");

        assert!(names(&new).is_empty());
        assert_eq!(names(&library), ["Alpha", "Beta", "Notes", "loose.txt"]);
    }

    #[test]
    fn a_read_only_file_is_moved_and_removed_from_the_source() {
        let scratch = Scratch::new("move-readonly");
        let (library, new) = move_scene(&scratch);
        let locked = library.join("locked.txt");
        fs::write(&locked, b"locked").expect("the file can be written");
        let mut permissions = fs::metadata(&locked).expect("the file has metadata").permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&locked, permissions).expect("the flag can be set");
        let ops = TestOps {
            fail_rename: Some(0..usize::MAX),
            ..TestOps::default()
        };

        let done = move_library(&library, &new, &ops).expect("the move succeeds");
        let finished = finish_move(done, &ops);

        assert_eq!(finished.failed, 0);
        assert!(new.join("locked.txt").is_file());
        assert!(!library.exists());
    }

    #[test]
    fn a_link_is_never_followed_and_stays_in_the_source_when_the_copy_is_used() {
        let scratch = Scratch::new("move-link");
        let (library, new) = move_scene(&scratch);
        fs::write(scratch.path("outside.txt"), b"outside").expect("the file can be written");
        if !link(&scratch.path("outside.txt"), &library.join("link.txt")) {
            return;
        }
        let ops = TestOps {
            fail_rename: Some(0..usize::MAX),
            ..TestOps::default()
        };

        let done = move_library(&library, &new, &ops).expect("the move succeeds");
        let finished = finish_move(done, &ops);

        assert_eq!(
            names(&new),
            ["Alpha", "Beta", "Notes", "loose.txt"],
            "the link is not copied"
        );
        assert_eq!(finished.removed, 4);
        assert_eq!(
            names(&library),
            ["link.txt"],
            "the link stays, so does the folder around it"
        );
        assert!(scratch.path("outside.txt").is_file());
    }

    #[test]
    fn record_names_adds_each_name_once() {
        let mut list = vec!["A".to_owned()];

        record_names(&mut list, &["B".to_owned(), "A".to_owned(), "B".to_owned()]);

        assert_eq!(list, ["A", "B"]);
    }
}
