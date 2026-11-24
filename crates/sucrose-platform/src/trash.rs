//! The OS recycle bin (Windows) or trash (macOS, Linux), through the `trash` crate (D-09). Windows reaches the bin through
//! COM, so callers use it on a worker thread, never on a UI thread.

use std::path::Path;

use crate::PlatformError;

/// Moves `path` (a file or a folder) to the recycle bin or trash, so the user can restore it. The error text names no
/// path: callers log it.
pub fn move_to_trash(path: &Path) -> Result<(), PlatformError> {
    ::trash::delete(path).map_err(|error| PlatformError::Failed {
        context: "moving to the bin",
        message: match error {
            ::trash::Error::Os { code, .. } => format!("OS error {code}"),
            ::trash::Error::TargetedRoot => "the item is a root folder".to_owned(),
            ::trash::Error::CouldNotAccess { .. } => "the item does not exist or cannot be accessed".to_owned(),
            _ => "the bin refused the item".to_owned(),
        },
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::move_to_trash;

    #[test]
    fn a_missing_path_is_an_error_that_names_no_path() {
        let missing = std::env::temp_dir().join(format!("sucrose-no-such-trash-{}", std::process::id()));

        let error = move_to_trash(&missing).expect_err("a missing path cannot be moved");

        assert!(!error.to_string().contains("sucrose-no-such-trash"), "{error}");
    }

    /// Moves a temporary folder to the real bin; run by hand (`cargo test -p sucrose-platform -- --ignored`), it leaves one
    /// folder in the user's bin.
    #[test]
    #[ignore = "puts a temporary folder into the real recycle bin or trash"]
    fn a_temporary_folder_moves_to_the_real_bin() {
        let dir = std::env::temp_dir().join(format!("sucrose-trash-probe-{}", std::process::id()));
        fs::create_dir_all(dir.join("inner")).expect("the probe folder can be created");
        fs::write(dir.join("inner").join("file.txt"), b"probe").expect("the probe file can be written");

        move_to_trash(&dir).expect("the folder moves to the bin");

        assert!(!dir.exists(), "the folder is out of its place");
    }
}
