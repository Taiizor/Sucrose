//! Library maintenance on a worker thread of the service: the showcase themes are copied into the library once and, when
//! the user wants it, corrupt themes go to the recycle bin (LIB-01, LIB-02, D-06, D-09, D-10). The service decides
//! everything from its own settings and its own scan; the Portal only asks.

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use sucrose_config::model::Settings;
use sucrose_core::paths;
use sucrose_ipc::portal::MaintainReport;

/// What one maintenance run works on, copied from the settings so the worker never touches them.
#[derive(Debug, Clone)]
pub struct MaintainInput {
    /// The library folder; created when missing.
    pub location: PathBuf,
    /// The folder with the themes shipped for first use (`data_dir()/Showcase`), when the OS has a data folder.
    pub showcase: Option<PathBuf>,
    /// Folder names already recorded in `Warehouse.Showcase`.
    pub copied: Vec<String>,
}

impl MaintainInput {
    /// What the settings say about the library, with the showcase folder of this data folder.
    pub fn from_settings(settings: &Settings, legacy_root: &Path) -> Self {
        Self {
            location: settings.library_location(legacy_root),
            showcase: paths::showcase_dir(),
            copied: settings.warehouse.showcase.clone(),
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
#[allow(
    dead_code,
    reason = "the library move of the next plan copies across volumes with it"
)]
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

/// Runs the maintenance: the report and the showcase names to record in `Warehouse.Showcase`.
pub fn maintain(input: &MaintainInput) -> (MaintainReport, Vec<String>) {
    maintain_with(input, &|source, destination| fs::copy(source, destination))
}

/// [`maintain`] with the file copy given.
fn maintain_with(input: &MaintainInput, copy_file: CopyFile<'_>) -> (MaintainReport, Vec<String>) {
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
    (report, names)
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
        }
    }

    #[test]
    fn copies_only_the_showcase_folders_that_are_not_listed() {
        let scratch = Scratch::new("unlisted");
        theme(&scratch.path("Showcase"), "A");
        theme(&scratch.path("Showcase"), "B");

        let (report, names) = maintain(&input(&scratch, &["A"]));

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

        let (report, names) = maintain(&input(&scratch, &["A", "B"]));

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

        let (report, names) = maintain(&input(&scratch, &[]));

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

        let (report, names) = maintain(&input(&scratch, &[]));

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

        let (report, names) = maintain_with(&input(&scratch, &[]), &failing);

        assert!(names.is_empty(), "a failed copy is not recorded");
        assert_eq!(report.showcase_copied, 0);
        assert!(!scratch.path("Library/B").exists(), "the half copy is removed");
    }

    #[test]
    fn record_names_adds_each_name_once() {
        let mut list = vec!["A".to_owned()];

        record_names(&mut list, &["B".to_owned(), "A".to_owned(), "B".to_owned()]);

        assert_eq!(list, ["A", "B"]);
    }
}
