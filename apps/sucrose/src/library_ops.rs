//! Library maintenance on a worker thread of the service: the showcase themes are copied into the library once and, when
//! the user wants it, corrupt themes go to the recycle bin (LIB-01, LIB-02, D-06, D-09, D-10). The service decides
//! everything from its own settings and its own scan; the Portal only asks.

use std::{
    io,
    path::{Path, PathBuf},
};

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
    let _ = (from, to);
    Ok(CopyStats::default())
}

/// Runs the maintenance: the report and the showcase names to record in `Warehouse.Showcase`.
pub fn maintain(input: &MaintainInput) -> (MaintainReport, Vec<String>) {
    let _ = input;
    (
        MaintainReport {
            showcase_copied: 0,
            corrupt_moved: 0,
            corrupt_skipped: 0,
        },
        Vec::new(),
    )
}

/// Adds `names` to the recorded showcase list, each once.
pub fn record_names(list: &mut Vec<String>, names: &[String]) {
    let _ = (list, names);
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
    fn record_names_adds_each_name_once() {
        let mut list = vec!["A".to_owned()];

        record_names(&mut list, &["B".to_owned(), "A".to_owned(), "B".to_owned()]);

        assert_eq!(list, ["A", "B"]);
    }
}
