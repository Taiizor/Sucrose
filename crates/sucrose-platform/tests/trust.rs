//! Installer trust (ENG-06, D-15, T-04-41): a Microsoft-signed installer passes, a tampered copy, an unsigned file and a
//! missing file are refused. The signed sample is the real Visual C++ bootstrapper: point `SUCROSE_TRUST_SAMPLE` at a
//! downloaded `vc_redist.x64.exe` (the tests that need it say so and return when it is unset).

#![cfg(windows)]
// The skip message of the sample tests has to reach the test output (the plan's verify step greps for it).
#![allow(clippy::print_stdout)]

use std::path::{Path, PathBuf};

use sucrose_platform::TrustError;

/// A scratch folder of its own, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sucrose-trust-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the scratch folder can be created");
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The signed sample, or `None` (with a message) when the variable is unset.
fn sample() -> Option<PathBuf> {
    let Some(path) = std::env::var_os("SUCROSE_TRUST_SAMPLE") else {
        println!("SKIPPED: SUCROSE_TRUST_SAMPLE is not set, so the Microsoft sample test did not run");
        return None;
    };
    let path = PathBuf::from(path);
    assert!(
        path.is_file(),
        "SUCROSE_TRUST_SAMPLE does not name a file: {}",
        path.display()
    );
    Some(path)
}

fn verify(path: &Path) -> Result<sucrose_platform::SignerInfo, TrustError> {
    sucrose_platform::current().verify_microsoft_signature(path)
}

#[test]
fn a_microsoft_signed_installer_is_accepted() {
    let Some(sample) = sample() else { return };

    let signer = verify(&sample).expect("the Microsoft bootstrapper passes");

    assert_eq!(signer.subject, "Microsoft Corporation");
}

#[test]
fn a_copy_with_one_byte_changed_is_refused() {
    let Some(sample) = sample() else { return };
    let scratch = Scratch::new("tampered");
    let copy = scratch.0.join("vc_redist.x64.exe");
    let mut bytes = std::fs::read(&sample).expect("the sample can be read");
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0xFF;
    std::fs::write(&copy, &bytes).expect("the copy can be written");

    let verdict = verify(&copy);

    assert!(
        matches!(verdict, Err(TrustError::BadSignature | TrustError::Untrusted { .. })),
        "a tampered installer must be refused, got {verdict:?}"
    );
}

#[test]
fn an_unsigned_file_is_refused() {
    let scratch = Scratch::new("unsigned");
    let file = scratch.0.join("plain.exe");
    std::fs::write(&file, b"MZ this is not a signed program").expect("the file can be written");

    assert_eq!(verify(&file), Err(TrustError::NotSigned));
}

#[test]
fn a_missing_file_is_an_io_error() {
    let scratch = Scratch::new("missing");

    let verdict = verify(&scratch.0.join("nothing-here.exe"));

    assert!(matches!(verdict, Err(TrustError::Io(_))), "got {verdict:?}");
}
