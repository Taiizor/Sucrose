//! `cargo xtask fetch-engines`: download the pinned libmpv archive, check its SHA256, extract the DLL and
//! header, and place the DLL next to the development executables (D-016, D-028).
//!
//! The pins live in `xtask/engines.toml`. Nothing is written outside `target/`, and nothing from the archive is
//! trusted before its hash matches: a mismatch deletes the file and fails.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use serde::Deserialize;
use sha2::{Digest, Sha256};

/// The pin manifest, compiled in so the task works from any working directory.
const MANIFEST: &str = include_str!("../engines.toml");
/// Architectures the manifest may pin (Windows targets, D-027).
const ARCHES: [&str; 3] = ["x86_64", "i686", "aarch64"];
/// Written into the cache folder once the archive is verified and extracted.
const MARKER: &str = ".verified";

/// One pinned engine archive from `engines.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Pin {
    name: String,
    arch: String,
    url: String,
    sha256: String,
    size: u64,
    file: String,
    header: String,
    upstream_repo: String,
    upstream_tag: String,
    upstream_commit: String,
    mpv_version: String,
    client_api: String,
    license: String,
}

#[derive(Deserialize)]
struct Manifest {
    engine: Vec<Pin>,
}

impl Pin {
    /// The archive file name, the last segment of the URL.
    fn asset(&self) -> &str {
        self.url.rsplit('/').next().unwrap_or(&self.url)
    }

    fn describe(&self) -> String {
        format!(
            "{} {} (client API {}, {} {} @ {}, {})",
            self.name,
            self.mpv_version,
            self.client_api,
            self.upstream_repo,
            self.upstream_tag,
            self.upstream_commit,
            self.license
        )
    }
}

/// Parses and checks the pin manifest: known architectures once each, HTTPS URLs, 64-digit hashes, safe file names.
pub fn parse_manifest(text: &str) -> Result<Vec<Pin>, String> {
    let manifest: Manifest = toml::from_str(text).map_err(|e| format!("engines.toml: {e}"))?;
    let mut seen: Vec<&str> = Vec::new();
    for pin in &manifest.engine {
        let arch = pin.arch.as_str();
        if !ARCHES.contains(&arch) {
            return Err(format!("engines.toml: unknown arch '{arch}'"));
        }
        if seen.contains(&arch) {
            return Err(format!("engines.toml: arch '{arch}' is pinned twice"));
        }
        seen.push(arch);
        if !pin.url.starts_with("https://") || pin.asset().is_empty() {
            return Err(format!(
                "engines.toml: {arch}: url must be an https URL ending in the archive name"
            ));
        }
        let hash = pin.sha256.as_str();
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(format!("engines.toml: {arch}: sha256 must be 64 lowercase hex digits"));
        }
        for path in [&pin.file, &pin.header] {
            let safe = !path.is_empty()
                && Path::new(path).components().all(|c| matches!(c, Component::Normal(_)))
                && !path.contains('\\');
            if !safe {
                return Err(format!("engines.toml: {arch}: '{path}' must be a plain relative path"));
            }
        }
    }
    Ok(manifest.engine)
}

/// Hashes the whole file and compares it with the expected lowercase hex SHA256.
pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let name = path
        .file_name()
        .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
    let mut file = File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1 << 16];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual: String = hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!("sha256 mismatch for {name}: expected {expected}, got {actual}"))
    }
}

/// Maps a `std::env::consts::ARCH` value to the architecture names used in the manifest.
fn arch_for(consts_arch: &str) -> Option<&'static str> {
    match consts_arch {
        "x86_64" => Some("x86_64"),
        "x86" => Some("i686"),
        "aarch64" => Some("aarch64"),
        _ => None,
    }
}

/// The manifest architecture of the machine running xtask.
fn host_arch() -> Option<&'static str> {
    arch_for(std::env::consts::ARCH)
}

/// The Rust target triple whose build folder holds the executables of an architecture.
fn triple(arch: &str) -> &'static str {
    match arch {
        "i686" => "i686-pc-windows-msvc",
        "aarch64" => "aarch64-pc-windows-msvc",
        _ => "x86_64-pc-windows-msvc",
    }
}

/// Folders that should receive the DLL: `target/debug` (always) and `target/release` (when it exists) for the host
/// architecture, and `target/<triple>/{debug,release}` where they already exist.
fn destinations(target_dir: &Path, arch: &str, host: Option<&str>) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    if host == Some(arch) {
        folders.push(target_dir.join("debug"));
        let release = target_dir.join("release");
        if release.is_dir() {
            folders.push(release);
        }
    }
    for profile in ["debug", "release"] {
        let folder = target_dir.join(triple(arch)).join(profile);
        if folder.is_dir() {
            folders.push(folder);
        }
    }
    folders
}

/// Command line of the task.
struct Options {
    /// `None` means the host architecture; `Some("all")` every pinned one.
    arch: Option<String>,
    from: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options { arch: None, from: None };
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let mut value = |name: &str| rest.next().cloned().ok_or_else(|| format!("{name} needs a value"));
        match flag.as_str() {
            "--arch" => options.arch = Some(value("--arch")?),
            "--from" => options.from = Some(PathBuf::from(value("--from")?)),
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    Ok(options)
}

/// `cargo xtask fetch-engines [--arch x86_64|i686|aarch64|all] [--from <archive>]`.
pub fn run(args: &[String]) -> ExitCode {
    match fetch(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn fetch(args: &[String]) -> Result<(), String> {
    let options = parse_args(args)?;
    let pins = parse_manifest(MANIFEST)?;
    let host = host_arch();
    let wanted: Vec<&Pin> = match options.arch.as_deref() {
        Some("all") => pins.iter().collect(),
        Some(arch) => vec![
            pins.iter()
                .find(|p| p.arch == arch)
                .ok_or_else(|| format!("no pin for arch '{arch}'"))?,
        ],
        None => {
            let arch = host.ok_or("this machine's architecture has no pin; pass --arch")?;
            vec![
                pins.iter()
                    .find(|p| p.arch == arch)
                    .ok_or_else(|| format!("no pin for arch '{arch}'"))?,
            ]
        }
    };
    if options.from.is_some() && wanted.len() != 1 {
        return Err("--from needs a single --arch".to_owned());
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("cannot find the workspace root")?;
    let target_dir = std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    for pin in wanted {
        fetch_arch(pin, options.from.as_deref(), &target_dir, host)?;
    }
    Ok(())
}

fn fetch_arch(pin: &Pin, from: Option<&Path>, target_dir: &Path, host: Option<&str>) -> Result<(), String> {
    println!("{}: {}", pin.arch, pin.describe());
    let cache = target_dir.join("engines").join(&pin.arch);
    fs::create_dir_all(&cache).map_err(|e| format!("cannot create {}: {e}", cache.display()))?;
    let dll = cache.join(&pin.file);
    let header = cache.join(&pin.header);
    let marker = cache.join(MARKER);

    let current = fs::read_to_string(&marker).is_ok_and(|text| text.trim() == pin.sha256);
    if current && dll.is_file() && header.is_file() {
        println!("{}: up to date ({})", pin.arch, pin.sha256);
    } else {
        // A failed run must not leave a marker that vouches for half-extracted files.
        let _ = fs::remove_file(&marker);
        let archive = cache.join(pin.asset());
        obtain_archive(pin, from, &archive)?;
        extract_pinned(&archive, pin, &cache)?;
        fs::write(&marker, &pin.sha256).map_err(|e| format!("cannot write {}: {e}", marker.display()))?;
    }

    for folder in destinations(target_dir, &pin.arch, host) {
        fs::create_dir_all(&folder).map_err(|e| format!("cannot create {}: {e}", folder.display()))?;
        let placed = folder.join(&pin.file);
        fs::copy(&dll, &placed).map_err(|e| {
            format!(
                "cannot place {} (is a Sucrose process still running?): {e}",
                placed.display()
            )
        })?;
        println!("{}: placed {}", pin.arch, placed.display());
    }
    Ok(())
}

/// Puts a verified archive at `archive`: reuses a verified one, copies `from`, or downloads it.
fn obtain_archive(pin: &Pin, from: Option<&Path>, archive: &Path) -> Result<(), String> {
    if let Some(source) = from {
        let same = matches!((source.canonicalize(), archive.canonicalize()), (Ok(a), Ok(b)) if a == b);
        if !same {
            fs::copy(source, archive).map_err(|e| format!("cannot copy {}: {e}", source.display()))?;
        }
    } else if archive.is_file() && verify_sha256(archive, &pin.sha256).is_ok() {
        println!("{}: reusing {}", pin.arch, archive.display());
        return Ok(());
    } else {
        download(&pin.url, pin.size, archive)?;
    }
    // The hash is checked before the archive is opened as 7z (T-02-01); a bad file is removed.
    verify_sha256(archive, &pin.sha256).inspect_err(|_| {
        let _ = fs::remove_file(archive);
    })?;
    println!("{}: sha256 ok", pin.arch);
    Ok(())
}

fn download(url: &str, size: u64, dest: &Path) -> Result<(), String> {
    println!("downloading {url} ({} MiB)", size >> 20);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .build()
        .into();
    let response = agent
        .get(url)
        .call()
        .map_err(|e| format!("download of {url} failed: {e}"))?;
    let mut body = response.into_body();
    let mut reader = body.with_config().limit(size).reader();

    let partial = dest.with_extension("part");
    let mut out = File::create(&partial).map_err(|e| format!("cannot create {}: {e}", partial.display()))?;
    let copied = io::copy(&mut reader, &mut out).map_err(|e| format!("download of {url} failed: {e}"));
    let flushed = out
        .flush()
        .map_err(|e| format!("cannot write {}: {e}", partial.display()));
    drop(out);
    let outcome = copied.and_then(|bytes| {
        flushed?;
        if bytes == size {
            Ok(())
        } else {
            Err(format!("download of {url} ended after {bytes} of {size} bytes"))
        }
    });
    match outcome {
        Ok(()) => fs::rename(&partial, dest).map_err(|e| format!("cannot move {} into place: {e}", partial.display())),
        Err(message) => {
            let _ = fs::remove_file(&partial);
            Err(message)
        }
    }
}

/// Extracts only `pin.file` and `pin.header` from the archive, to `<out>/<file>` and `<out>/<header>`.
/// Destinations never come from the archive (T-02-03).
fn extract_pinned(archive: &Path, pin: &Pin, out: &Path) -> Result<(), String> {
    let source = File::open(archive).map_err(|e| format!("cannot open {}: {e}", archive.display()))?;
    let mut reader = sevenz_rust2::ArchiveReader::new(source, sevenz_rust2::Password::empty())
        .map_err(|e| format!("cannot read {}: {e}", archive.display()))?;
    let wanted = [
        (pin.file.as_str(), out.join(&pin.file)),
        (pin.header.as_str(), out.join(&pin.header)),
    ];
    let mut written = [false; 2];
    let mut problem: Option<String> = None;

    let walked = reader.for_each_entries(|entry, data| {
        if entry.is_directory() || !entry.has_stream() {
            return Ok(true);
        }
        let name = entry.name().replace('\\', "/");
        let Some(index) = wanted
            .iter()
            .position(|(suffix, _)| name == *suffix || name.ends_with(&format!("/{suffix}")))
        else {
            return Ok(true);
        };
        if written[index] {
            problem = Some(format!(
                "{} holds more than one entry named {}",
                archive.display(),
                wanted[index].0
            ));
            return Ok(false);
        }
        let destination = &wanted[index].1;
        let temporary = destination.with_extension("tmp");
        let stored = (|| -> io::Result<()> {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(&temporary)?;
            io::copy(data, &mut file)?;
            file.flush()?;
            drop(file);
            fs::rename(&temporary, destination)
        })();
        match stored {
            Ok(()) => {
                written[index] = true;
                Ok(true)
            }
            Err(e) => {
                let _ = fs::remove_file(&temporary);
                problem = Some(format!("cannot write {}: {e}", destination.display()));
                Ok(false)
            }
        }
    });
    walked.map_err(|e| format!("cannot extract {}: {e}", archive.display()))?;
    if let Some(message) = problem {
        return Err(message);
    }
    for (index, (suffix, _)) in wanted.iter().enumerate() {
        if !written[index] {
            return Err(format!("{} has no entry named {suffix}", archive.display()));
        }
    }
    println!("{}: extracted {} and {}", pin.arch, pin.file, pin.header);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch folder removed on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("sucrose-xtask-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create scratch folder");
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const HELLO_SHA256: &str = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

    fn test_pin(arch: &str) -> Pin {
        let mut pins = parse_manifest(MANIFEST).expect("manifest parses");
        let mut pin = pins.remove(0);
        pin.arch = arch.to_owned();
        pin
    }

    #[test]
    fn the_committed_manifest_pins_three_architectures() {
        let pins = parse_manifest(MANIFEST).expect("manifest parses");
        let arches: Vec<&str> = pins.iter().map(|p| p.arch.as_str()).collect();
        assert_eq!(arches, ["x86_64", "i686", "aarch64"]);
        for pin in &pins {
            assert!(
                pin.url
                    .starts_with("https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20261111/")
            );
            assert!(
                !pin.url.contains("x86_64-v3"),
                "the v3 build needs AVX2 and is never pinned"
            );
            assert_eq!(pin.file, "libmpv-2.dll");
            assert_eq!(pin.header, "include/mpv/client.h");
        }
        assert_eq!(
            pins[0].sha256,
            "273c079b11ea5575822f972756f4c9a71149a5c10130353abeb8586738938452"
        );
    }

    #[test]
    fn parse_manifest_rejects_bad_pins() {
        let good = MANIFEST;
        let bad_hash = good.replacen(
            "273c079b11ea5575822f972756f4c9a71149a5c10130353abeb8586738938452",
            "abc",
            1,
        );
        assert!(parse_manifest(&bad_hash).is_err());
        let plain_http = good.replacen("https://", "http://", 1);
        assert!(parse_manifest(&plain_http).is_err());
        let unknown_arch = good.replacen("arch = \"x86_64\"", "arch = \"riscv\"", 1);
        assert!(parse_manifest(&unknown_arch).is_err());
        let duplicate = good.replacen("arch = \"i686\"", "arch = \"x86_64\"", 1);
        assert!(parse_manifest(&duplicate).is_err());
        let escaping = good.replacen("file = \"libmpv-2.dll\"", "file = \"../libmpv-2.dll\"", 1);
        assert!(parse_manifest(&escaping).is_err());
    }

    #[test]
    fn verify_sha256_accepts_the_right_hash() {
        let scratch = Scratch::new("hash-ok");
        let file = scratch.0.join("hello.bin");
        fs::write(&file, b"hello world").expect("write");
        assert_eq!(verify_sha256(&file, HELLO_SHA256), Ok(()));
    }

    #[test]
    fn verify_sha256_rejects_a_wrong_hash() {
        let scratch = Scratch::new("hash-bad");
        let file = scratch.0.join("hello.bin");
        fs::write(&file, b"hello world").expect("write");
        let wrong = "0".repeat(64);
        let error = verify_sha256(&file, &wrong).expect_err("a wrong hash must fail");
        assert!(error.starts_with("sha256 mismatch for hello.bin"), "{error}");
    }

    #[test]
    fn verify_sha256_rejects_a_missing_file() {
        let scratch = Scratch::new("hash-missing");
        assert!(verify_sha256(&scratch.0.join("absent.bin"), HELLO_SHA256).is_err());
    }

    #[test]
    fn a_mismatching_archive_is_deleted_and_never_extracted() {
        let scratch = Scratch::new("tampered");
        let archive = scratch.0.join("fake.7z");
        fs::write(&archive, b"not an archive").expect("write");
        let pin = test_pin("x86_64");
        let error = obtain_archive(&pin, Some(&archive), &scratch.0.join("cache.7z")).expect_err("hash must not match");
        assert!(error.starts_with("sha256 mismatch"), "{error}");
        assert!(archive.exists(), "the user's own --from file is never deleted");
        assert!(!scratch.0.join("cache.7z").exists(), "the copy in the cache is deleted");
    }

    #[test]
    fn arch_for_maps_rust_names_to_pin_names() {
        assert_eq!(arch_for("x86_64"), Some("x86_64"));
        assert_eq!(arch_for("x86"), Some("i686"));
        assert_eq!(arch_for("aarch64"), Some("aarch64"));
        assert_eq!(arch_for("riscv64"), None);
    }

    #[test]
    fn destinations_always_include_debug_for_the_host_only() {
        let scratch = Scratch::new("destinations");
        let target = &scratch.0;
        assert_eq!(destinations(target, "x86_64", Some("x86_64")), [target.join("debug")]);
        assert!(destinations(target, "i686", Some("x86_64")).is_empty());

        fs::create_dir_all(target.join("release")).expect("create");
        fs::create_dir_all(target.join("i686-pc-windows-msvc").join("debug")).expect("create");
        assert_eq!(
            destinations(target, "x86_64", Some("x86_64")),
            [target.join("debug"), target.join("release")]
        );
        assert_eq!(
            destinations(target, "i686", Some("x86_64")),
            [target.join("i686-pc-windows-msvc").join("debug")]
        );
    }

    #[test]
    fn parse_args_reads_arch_and_from() {
        let args: Vec<String> = ["--arch", "all", "--from", "a.7z"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let options = parse_args(&args).expect("valid");
        assert_eq!(options.arch.as_deref(), Some("all"));
        assert_eq!(options.from, Some(PathBuf::from("a.7z")));
        assert!(parse_args(&["--arch".to_owned()]).is_err());
        assert!(parse_args(&["--bogus".to_owned()]).is_err());
    }

    #[test]
    fn extraction_writes_only_the_two_named_entries_to_fixed_paths() {
        let scratch = Scratch::new("extract");
        let content = scratch.0.join("content");
        fs::create_dir_all(content.join("pkg").join("include").join("mpv")).expect("create");
        fs::write(content.join("pkg").join("libmpv-2.dll"), b"dll bytes").expect("write");
        fs::write(
            content.join("pkg").join("include").join("mpv").join("client.h"),
            b"header bytes",
        )
        .expect("write");
        fs::write(content.join("pkg").join("other.txt"), b"unrelated").expect("write");
        let archive = scratch.0.join("fixture.7z");
        sevenz_rust2::compress_to_path(&content, &archive).expect("build fixture archive");

        let out = scratch.0.join("out");
        let pin = test_pin("x86_64");
        extract_pinned(&archive, &pin, &out).expect("extracts");
        assert_eq!(fs::read(out.join("libmpv-2.dll")).expect("dll"), b"dll bytes");
        assert_eq!(
            fs::read(out.join("include").join("mpv").join("client.h")).expect("header"),
            b"header bytes"
        );
        assert!(!out.join("other.txt").exists());
        assert!(!out.join("pkg").exists(), "archive paths are never reproduced");
    }

    #[test]
    fn extraction_reports_a_missing_entry() {
        let scratch = Scratch::new("extract-missing");
        let content = scratch.0.join("content");
        fs::create_dir_all(&content).expect("create");
        fs::write(content.join("libmpv-2.dll"), b"dll bytes").expect("write");
        let archive = scratch.0.join("fixture.7z");
        sevenz_rust2::compress_to_path(&content, &archive).expect("build fixture archive");

        let error =
            extract_pinned(&archive, &test_pin("x86_64"), &scratch.0.join("out")).expect_err("no header inside");
        assert!(error.contains("include/mpv/client.h"), "{error}");
    }
}
