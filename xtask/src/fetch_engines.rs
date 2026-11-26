//! `cargo xtask fetch-engines`: download the pinned engine archives, check their SHA256, and put the files where the
//! builds read them (D-016, D-028, D-32).
//!
//! libmpv: extract the DLL and header, and place the DLL next to the development executables. CEF: extract the
//! whole dist as a tree under `target/engines/cef/<cef_version>/<os_arch>/`, the folder `cef-dll-sys` reads through
//! `CEF_PATH` (`.cargo/config.toml`), so no build script ever downloads CEF.
//!
//! The pins live in `xtask/engines.toml`. Nothing is written outside `target/`, and nothing from an archive is
//! trusted before its hash matches: a mismatch deletes the file and fails.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use bzip2::read::BzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The pin manifest, compiled in so the task works from any working directory.
const MANIFEST: &str = include_str!("../engines.toml");
/// Architectures the manifest may pin (Windows targets, D-027).
const ARCHES: [&str; 3] = ["x86_64", "i686", "aarch64"];
/// Engines the manifest may pin.
const ENGINES: [&str; 2] = ["libmpv", "cef"];
/// Written into the cache folder once the archive is verified and extracted.
const MARKER: &str = ".verified";
/// The CEF files whose absence means the extracted tree is not a usable dist.
const CEF_REQUIRED: [&str; 4] = [
    "libcef.dll",
    "libcef.lib",
    "CMakeLists.txt",
    "include/cef_api_versions.h",
];
/// Top-level entries of the CEF archive that are copied as they are (the rest is dropped).
const CEF_TOP_ENTRIES: [&str; 7] = [
    "CMakeLists.txt",
    "CREDITS.html",
    "LICENSE.txt",
    "README.txt",
    "cmake",
    "include",
    "libcef_dll",
];

/// One pinned engine archive from `engines.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Pin {
    name: String,
    arch: String,
    url: String,
    sha256: String,
    size: u64,
    license: String,
    // libmpv rows
    file: Option<String>,
    header: Option<String>,
    upstream_repo: Option<String>,
    upstream_tag: Option<String>,
    upstream_commit: Option<String>,
    mpv_version: Option<String>,
    client_api: Option<String>,
    // cef rows
    version: Option<String>,
    upstream_sha1: Option<String>,
    layout: Option<String>,
}

#[derive(Deserialize)]
struct Manifest {
    engine: Vec<Pin>,
}

/// The `archive.json` that `download-cef` (and so `cef-dll-sys`) reads to check the dist version.
#[derive(Serialize)]
struct ArchiveJson<'a> {
    #[serde(rename = "type")]
    file_type: &'a str,
    name: &'a str,
    sha1: &'a str,
}

fn text(field: &Option<String>) -> &str {
    field.as_deref().unwrap_or_default()
}

impl Pin {
    /// The archive file name, the last segment of the URL.
    fn asset(&self) -> &str {
        self.url.rsplit('/').next().unwrap_or(&self.url)
    }

    fn is_cef(&self) -> bool {
        self.name == "cef"
    }

    /// The libmpv DLL name inside the archive.
    fn file(&self) -> &str {
        text(&self.file)
    }

    /// The libmpv header path inside the archive.
    fn header(&self) -> &str {
        text(&self.header)
    }

    fn describe(&self) -> String {
        if self.is_cef() {
            return format!(
                "cef {} (sha1 {}, {})",
                text(&self.version),
                text(&self.upstream_sha1),
                self.license
            );
        }
        format!(
            "{} {} (client API {}, {} {} @ {}, {})",
            self.name,
            text(&self.mpv_version),
            text(&self.client_api),
            text(&self.upstream_repo),
            text(&self.upstream_tag),
            text(&self.upstream_commit),
            self.license
        )
    }

    /// The CEF archive's real file name: the URL spells `+` as `%2B`.
    fn cef_archive_name(&self) -> String {
        self.asset().replace("%2B", "+").replace("%2b", "+")
    }

    /// The single folder every entry of the CEF archive lives in: the archive name without `.tar.bz2`.
    fn cef_top_folder(&self) -> String {
        let name = self.cef_archive_name();
        name.strip_suffix(".tar.bz2").map_or(name.clone(), str::to_owned)
    }

    /// The version folder `cef-dll-sys` joins to `CEF_PATH`: the crate's build metadata, `156.0.31`.
    fn cef_version_folder(&self) -> &str {
        text(&self.version).split('+').next().unwrap_or_default()
    }

    /// The os/arch folder `download-cef` names the dist after (`OsAndArch`), e.g. `cef_windows_x86_64`.
    fn cef_os_arch(&self) -> &'static str {
        match self.arch.as_str() {
            "i686" => "cef_windows_x86",
            "aarch64" => "cef_windows_aarch64",
            _ => "cef_windows_x86_64",
        }
    }
}

/// Parses and checks the pin manifest: known engines and architectures once each, HTTPS URLs, hashes, the fields each
/// engine needs, safe file names.
pub fn parse_manifest(text_in: &str) -> Result<Vec<Pin>, String> {
    let manifest: Manifest = toml::from_str(text_in).map_err(|e| format!("engines.toml: {e}"))?;
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for pin in &manifest.engine {
        let (name, arch) = (pin.name.as_str(), pin.arch.as_str());
        if !ENGINES.contains(&name) {
            return Err(format!("engines.toml: unknown engine '{name}'"));
        }
        if !ARCHES.contains(&arch) {
            return Err(format!("engines.toml: unknown arch '{arch}'"));
        }
        if seen.contains(&(name, arch)) {
            return Err(format!("engines.toml: {name} arch '{arch}' is pinned twice"));
        }
        seen.push((name, arch));
        if !pin.url.starts_with("https://") || pin.asset().is_empty() {
            return Err(format!(
                "engines.toml: {name} {arch}: url must be an https URL ending in the archive name"
            ));
        }
        if !is_lower_hex(&pin.sha256, 64) {
            return Err(format!(
                "engines.toml: {name} {arch}: sha256 must be 64 lowercase hex digits"
            ));
        }
        if pin.size == 0 {
            return Err(format!(
                "engines.toml: {name} {arch}: size must be the archive size in bytes"
            ));
        }
        if name == "cef" {
            if text(&pin.version).is_empty() {
                return Err(format!("engines.toml: cef {arch}: version is required"));
            }
            if !is_lower_hex(text(&pin.upstream_sha1), 40) {
                return Err(format!(
                    "engines.toml: cef {arch}: upstream_sha1 must be 40 lowercase hex digits"
                ));
            }
            if pin.layout.as_deref() != Some("tree") {
                return Err(format!("engines.toml: cef {arch}: layout must be \"tree\""));
            }
            if !pin.asset().ends_with(".tar.bz2") {
                return Err(format!("engines.toml: cef {arch}: url must name a .tar.bz2 archive"));
            }
            continue;
        }
        for (field, value) in [
            ("file", &pin.file),
            ("header", &pin.header),
            ("mpv_version", &pin.mpv_version),
            ("client_api", &pin.client_api),
            ("upstream_repo", &pin.upstream_repo),
            ("upstream_tag", &pin.upstream_tag),
            ("upstream_commit", &pin.upstream_commit),
        ] {
            if text(value).is_empty() {
                return Err(format!("engines.toml: libmpv {arch}: {field} is required"));
            }
        }
        for path in [pin.file(), pin.header()] {
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

fn is_lower_hex(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// One line naming the pinned build of an architecture, for provenance comments in generated files.
#[cfg(feature = "bindgen")]
pub fn pin_summary(arch: &str) -> Result<String, String> {
    let pins = parse_manifest(MANIFEST)?;
    pins.iter()
        .find(|p| p.name == "libmpv" && p.arch == arch)
        .map(Pin::describe)
        .ok_or_else(|| format!("no pin for arch '{arch}'"))
}

/// Hashes the whole file and compares it with the expected lowercase hex SHA256.
pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let name = path
        .file_name()
        .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
    let actual = file_sha256(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!("sha256 mismatch for {name}: expected {expected}, got {actual}"))
    }
}

/// The lowercase hex SHA256 of a whole file.
fn file_sha256(path: &Path) -> Result<String, String> {
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
    Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
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
    /// `None` means every pinned engine; otherwise one name of [`ENGINES`].
    engine: Option<String>,
    from: Option<PathBuf>,
    /// Download (or take `--from`), check the size, print the SHA256 and stop.
    print_sha256: bool,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        arch: None,
        engine: None,
        from: None,
        print_sha256: false,
    };
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let mut value = |name: &str| rest.next().cloned().ok_or_else(|| format!("{name} needs a value"));
        match flag.as_str() {
            "--arch" => options.arch = Some(value("--arch")?),
            "--engine" => {
                let engine = value("--engine")?;
                if engine != "all" && !ENGINES.contains(&engine.as_str()) {
                    return Err(format!("unknown engine '{engine}'; use libmpv, cef or all"));
                }
                options.engine = (engine != "all").then_some(engine);
            }
            "--from" => options.from = Some(PathBuf::from(value("--from")?)),
            "--print-sha256" => options.print_sha256 = true,
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    Ok(options)
}

/// `cargo xtask fetch-engines [--engine libmpv|cef|all] [--arch x86_64|i686|aarch64|all] [--from <archive>] [--print-sha256]`.
pub fn run(args: &[String]) -> ExitCode {
    match fetch(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("failed: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The pins a command line asks for: the chosen engine (default every one) for the chosen architecture (default the
/// host's, `all` for every pinned one).
fn select<'a>(pins: &'a [Pin], options: &Options, host: Option<&str>) -> Result<Vec<&'a Pin>, String> {
    let arch = match options.arch.as_deref() {
        Some("all") => None,
        Some(arch) => Some(arch),
        None => Some(host.ok_or("this machine's architecture has no pin; pass --arch")?),
    };
    let wanted: Vec<&Pin> = pins
        .iter()
        .filter(|p| options.engine.as_deref().is_none_or(|engine| p.name == engine))
        .filter(|p| arch.is_none_or(|arch| p.arch == arch))
        .collect();
    if wanted.is_empty() {
        return Err(format!(
            "no pin for engine '{}' and arch '{}'",
            options.engine.as_deref().unwrap_or("all"),
            arch.unwrap_or("all")
        ));
    }
    Ok(wanted)
}

fn fetch(args: &[String]) -> Result<(), String> {
    let options = parse_args(args)?;
    let pins = parse_manifest(MANIFEST)?;
    let host = host_arch();
    let wanted = select(&pins, &options, host)?;
    if options.from.is_some() && wanted.len() != 1 {
        return Err("--from needs a single --engine and a single --arch".to_owned());
    }

    let target_dir = crate::target_dir();
    for pin in wanted {
        if options.print_sha256 {
            print_sha256(pin, options.from.as_deref(), &target_dir)?;
        } else if pin.is_cef() {
            fetch_cef(pin, options.from.as_deref(), &target_dir)?;
        } else {
            fetch_libmpv(pin, options.from.as_deref(), &target_dir, host)?;
        }
    }
    Ok(())
}

/// Where the archive of a pin is cached: libmpv next to its extracted files, CEF in `target/engines/cef/downloads`
/// (outside every version folder, so `cef-dll-sys` never sees it).
fn archive_path(pin: &Pin, target_dir: &Path) -> PathBuf {
    if pin.is_cef() {
        target_dir
            .join("engines")
            .join("cef")
            .join("downloads")
            .join(pin.cef_archive_name())
    } else {
        target_dir.join("engines").join(&pin.arch).join(pin.asset())
    }
}

/// `--print-sha256`: the pinning step. Takes the archive from `--from`, a cached download of the right size, or the
/// pinned URL, and prints its SHA256 for `engines.toml`. The manifest is never edited by code.
fn print_sha256(pin: &Pin, from: Option<&Path>, target_dir: &Path) -> Result<(), String> {
    let cached = archive_path(pin, target_dir);
    let source = match from {
        Some(path) => path.to_path_buf(),
        None => {
            if let Some(parent) = cached.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            if fs::metadata(&cached).map_or(true, |meta| meta.len() != pin.size) {
                download(&pin.url, pin.size, &cached)?;
            }
            cached
        }
    };
    let actual = fs::metadata(&source)
        .map_err(|e| format!("cannot read {}: {e}", source.display()))?
        .len();
    if actual != pin.size {
        return Err(format!(
            "{} is {actual} bytes, the pin says {}",
            source.display(),
            pin.size
        ));
    }
    println!("{} {}: size ok ({actual} bytes)", pin.name, pin.arch);
    println!("{}  sha256 {}", source.display(), file_sha256(&source)?);
    Ok(())
}

fn fetch_libmpv(pin: &Pin, from: Option<&Path>, target_dir: &Path, host: Option<&str>) -> Result<(), String> {
    println!("{}: {}", pin.arch, pin.describe());
    let cache = target_dir.join("engines").join(&pin.arch);
    fs::create_dir_all(&cache).map_err(|e| format!("cannot create {}: {e}", cache.display()))?;
    let dll = cache.join(pin.file());
    let header = cache.join(pin.header());
    let marker = cache.join(MARKER);

    let current = fs::read_to_string(&marker).is_ok_and(|text| text.trim() == pin.sha256);
    if current && dll.is_file() && header.is_file() {
        println!("{}: up to date ({})", pin.arch, pin.sha256);
    } else {
        // A failed run must not leave a marker that vouches for half-extracted files.
        let _ = fs::remove_file(&marker);
        let archive = archive_path(pin, target_dir);
        obtain_archive(pin, from, &archive)?;
        extract_pinned(&archive, pin, &cache)?;
        fs::write(&marker, &pin.sha256).map_err(|e| format!("cannot write {}: {e}", marker.display()))?;
    }

    for folder in destinations(target_dir, &pin.arch, host) {
        fs::create_dir_all(&folder).map_err(|e| format!("cannot create {}: {e}", folder.display()))?;
        let placed = folder.join(pin.file());
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

/// The folder `cef-dll-sys` reads for a pin: `<CEF_PATH>/<cef_version>/<os_arch>`.
fn cef_dist_dir(pin: &Pin, target_dir: &Path) -> PathBuf {
    target_dir
        .join("engines")
        .join("cef")
        .join(pin.cef_version_folder())
        .join(pin.cef_os_arch())
}

/// Whether the pinned CEF dist of this host is in `target/engines/cef`. A host that does not build CEF counts as
/// fetched. `cargo xtask ci` asks it first, so the local gate never lets `cef-dll-sys` download CEF.
pub fn cef_fetched(target_dir: &Path) -> bool {
    if !cfg!(windows) {
        return true;
    }
    host_arch().is_none_or(|arch| dist_fetched(target_dir, arch))
}

fn dist_fetched(target_dir: &Path, arch: &str) -> bool {
    // The architectures the cef dependency is enabled for (crates/sucrose-engine-cef/Cargo.toml).
    if !matches!(arch, "x86_64" | "aarch64") {
        return true;
    }
    parse_manifest(MANIFEST).is_ok_and(|pins| {
        pins.iter()
            .filter(|pin| pin.is_cef() && pin.arch == arch)
            .all(|pin| cef_dist_dir(pin, target_dir).join("archive.json").is_file())
    })
}

/// Fetches, verifies and extracts the pinned CEF dist. The runtime files (DLLs, paks, `icudtl.dat`, `locales/`) are
/// not placed here: `cef-dll-sys` copies them next to the built executables.
fn fetch_cef(pin: &Pin, from: Option<&Path>, target_dir: &Path) -> Result<(), String> {
    println!("{}: {}", pin.arch, pin.describe());
    let dist = cef_dist_dir(pin, target_dir);
    let parent = dist
        .parent()
        .ok_or_else(|| format!("{} has no parent folder", dist.display()))?;
    let marker = parent.join(format!("{}.verified", pin.cef_os_arch()));

    let current = fs::read_to_string(&marker).is_ok_and(|text| text.trim() == pin.sha256);
    if current && dist.join("archive.json").is_file() && dist.join("libcef.dll").is_file() {
        println!("{}: up to date ({})", pin.arch, pin.sha256);
        return Ok(());
    }
    // A failed run must not leave a marker that vouches for a half-extracted dist.
    let _ = fs::remove_file(&marker);
    let archive = archive_path(pin, target_dir);
    if let Some(folder) = archive.parent() {
        fs::create_dir_all(folder).map_err(|e| format!("cannot create {}: {e}", folder.display()))?;
    }
    obtain_archive(pin, from, &archive)?;
    install_cef(&archive, pin, &dist)?;
    fs::write(&marker, &pin.sha256).map_err(|e| format!("cannot write {}: {e}", marker.display()))?;
    println!(
        "{}: CEF {} is in {}",
        pin.arch,
        pin.cef_version_folder(),
        dist.display()
    );
    Ok(())
}

/// Extracts the verified archive into `dist` and writes the `archive.json` `download-cef` expects there.
fn install_cef(archive: &Path, pin: &Pin, dist: &Path) -> Result<(), String> {
    extract_cef(archive, pin, dist)?;
    let name = pin.cef_archive_name();
    let json = serde_json::to_string_pretty(&ArchiveJson {
        file_type: "minimal",
        name: &name,
        sha1: text(&pin.upstream_sha1),
    })
    .map_err(|e| format!("cannot write archive.json: {e}"))?;
    let path = dist.join("archive.json");
    fs::write(&path, json).map_err(|e| format!("cannot write {}: {e}", path.display()))
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
    // The hash is checked before the archive is opened (T-02-01, T-04-01); a bad file is removed.
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

/// Extracts only the libmpv `file` and `header` from the archive, to `<out>/<file>` and `<out>/<header>`.
/// Destinations never come from the archive (T-02-03).
fn extract_pinned(archive: &Path, pin: &Pin, out: &Path) -> Result<(), String> {
    let source = File::open(archive).map_err(|e| format!("cannot open {}: {e}", archive.display()))?;
    let mut reader = sevenz_rust2::ArchiveReader::new(source, sevenz_rust2::Password::empty())
        .map_err(|e| format!("cannot read {}: {e}", archive.display()))?;
    let wanted = [
        (pin.file(), out.join(pin.file())),
        (pin.header(), out.join(pin.header())),
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
    println!("{}: extracted {} and {}", pin.arch, pin.file(), pin.header());
    Ok(())
}

/// A path from an archive as plain names: no root, drive, `..`, `:` or backslash in any part (T-04-02).
fn safe_parts(raw: &Path) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    for component in raw.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => {
                let part = part
                    .to_str()
                    .ok_or_else(|| format!("entry '{}' is not valid text", raw.display()))?;
                if part.contains(':') || part.contains('\\') {
                    return Err(format!("entry '{}' has an unsafe name", raw.display()));
                }
                parts.push(part.to_owned());
            }
            _ => return Err(format!("entry '{}' is absolute or leaves the archive", raw.display())),
        }
    }
    Ok(parts)
}

/// Extracts the CEF tar.bz2 into the layout `cef-dll-sys` reads (what `download-cef` makes): `Release/` and
/// `Resources/` merged into the root, the CMake files, `include/` and `libcef_dll/` beside them. Entries are
/// written to a staging folder and the folder is renamed to `dist` only when the whole archive is acceptable; an
/// absolute or `..` name, a link, a device or an entry outside the archive's single top folder refuses the archive
/// and leaves nothing behind (T-04-02).
fn extract_cef(archive: &Path, pin: &Pin, dist: &Path) -> Result<(), String> {
    let top = pin.cef_top_folder();
    let file = File::open(archive).map_err(|e| format!("cannot open {}: {e}", archive.display()))?;
    let mut tar = tar::Archive::new(BzDecoder::new(BufReader::new(file)));

    let staging = dist.with_file_name(format!("extracting_{}", pin.cef_os_arch()));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| format!("cannot clear {}: {e}", staging.display()))?;
    }
    fs::create_dir_all(&staging).map_err(|e| format!("cannot create {}: {e}", staging.display()))?;

    let outcome = extract_cef_entries(&mut tar, &top, &staging, archive);
    let checked = outcome.and_then(|()| {
        for required in CEF_REQUIRED {
            if !staging.join(required).is_file() {
                return Err(format!("{} has no {required}", archive.display()));
            }
        }
        Ok(())
    });
    if let Err(message) = checked {
        let _ = fs::remove_dir_all(&staging);
        return Err(message);
    }
    if dist.exists() {
        fs::remove_dir_all(dist).map_err(|e| format!("cannot replace {}: {e}", dist.display()))?;
    }
    fs::rename(&staging, dist).map_err(|e| format!("cannot move {} into place: {e}", staging.display()))
}

fn extract_cef_entries<R: Read>(
    tar: &mut tar::Archive<R>,
    top: &str,
    staging: &Path,
    archive: &Path,
) -> Result<(), String> {
    let mut written: HashSet<PathBuf> = HashSet::new();
    let entries = tar
        .entries()
        .map_err(|e| format!("cannot read {}: {e}", archive.display()))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("cannot read {}: {e}", archive.display()))?;
        let kind = entry.header().entry_type();
        let raw = entry
            .path()
            .map_err(|e| format!("{} has an unreadable entry name: {e}", archive.display()))?
            .into_owned();
        match kind {
            tar::EntryType::XGlobalHeader | tar::EntryType::XHeader => continue,
            tar::EntryType::Regular | tar::EntryType::Continuous | tar::EntryType::Directory => {}
            other => {
                return Err(format!(
                    "{} holds the entry '{}' of type {other:?}; links and devices are refused",
                    archive.display(),
                    raw.display()
                ));
            }
        }
        let parts = safe_parts(&raw).map_err(|e| format!("{}: {e}", archive.display()))?;
        let Some((first, rest)) = parts.split_first() else {
            continue;
        };
        if first != top {
            return Err(format!(
                "{}: entry '{}' is outside the archive's top folder '{top}'",
                archive.display(),
                raw.display()
            ));
        }
        let Some((root, inner)) = rest.split_first() else {
            if kind == tar::EntryType::Directory {
                continue;
            }
            return Err(format!("{}: the top folder name '{top}' is a file", archive.display()));
        };
        let relative: &[String] = match root.as_str() {
            "Release" | "Resources" => inner,
            name if CEF_TOP_ENTRIES.contains(&name) => rest,
            _ => continue,
        };
        if relative.is_empty() {
            continue;
        }
        let destination = relative
            .iter()
            .fold(staging.to_path_buf(), |path, part| path.join(part));
        if kind == tar::EntryType::Directory {
            fs::create_dir_all(&destination).map_err(|e| format!("cannot create {}: {e}", destination.display()))?;
            continue;
        }
        if !written.insert(destination.clone()) {
            return Err(format!(
                "{} holds two entries that land on {}",
                archive.display(),
                destination.display()
            ));
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        let mut out = File::create(&destination).map_err(|e| format!("cannot write {}: {e}", destination.display()))?;
        io::copy(&mut entry, &mut out)
            .and_then(|_| out.flush())
            .map_err(|e| format!("cannot write {}: {e}", destination.display()))?;
    }
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
        let all = parse_manifest(MANIFEST).expect("manifest parses");
        let pins: Vec<&Pin> = all.iter().filter(|p| p.name == "libmpv").collect();
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
            assert_eq!(pin.file(), "libmpv-2.dll");
            assert_eq!(pin.header(), "include/mpv/client.h");
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
        assert_eq!(options.engine, None);
        assert!(!options.print_sha256);
        assert!(parse_args(&["--arch".to_owned()]).is_err());
        assert!(parse_args(&["--bogus".to_owned()]).is_err());
    }

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parse_args_reads_engine_and_print_sha256() {
        let options = parse_args(&strings(&["--engine", "cef", "--print-sha256"])).expect("valid");
        assert_eq!(options.engine.as_deref(), Some("cef"));
        assert!(options.print_sha256);
        let all = parse_args(&strings(&["--engine", "all"])).expect("valid");
        assert_eq!(all.engine, None);
        assert!(parse_args(&strings(&["--engine", "vlc"])).is_err());
        assert!(parse_args(&strings(&["--engine"])).is_err());
    }

    #[test]
    fn select_picks_the_engine_and_architecture_asked_for() {
        let pins = parse_manifest(MANIFEST).expect("manifest parses");
        let names = |args: &[&str], host: Option<&str>| -> Vec<(String, String)> {
            let options = parse_args(&strings(args)).expect("valid");
            select(&pins, &options, host)
                .expect("selects")
                .iter()
                .map(|p| (p.name.clone(), p.arch.clone()))
                .collect()
        };
        // No argument: every engine pinned for the host.
        assert_eq!(
            names(&[], Some("x86_64")),
            [
                ("libmpv".to_owned(), "x86_64".to_owned()),
                ("cef".to_owned(), "x86_64".to_owned())
            ]
        );
        assert_eq!(
            names(&["--engine", "cef"], Some("aarch64")),
            [("cef".to_owned(), "aarch64".to_owned())]
        );
        assert_eq!(names(&["--engine", "libmpv", "--arch", "all"], None).len(), 3);
        assert_eq!(names(&["--arch", "all"], None).len(), 6);
        let nothing = parse_args(&strings(&["--arch", "riscv"])).expect("valid");
        assert!(select(&pins, &nothing, Some("x86_64")).is_err());
        let no_host = parse_args(&strings(&[])).expect("valid");
        assert!(select(&pins, &no_host, None).is_err());
    }

    fn cef_pin(arch: &str) -> Pin {
        parse_manifest(MANIFEST)
            .expect("manifest parses")
            .into_iter()
            .find(|p| p.name == "cef" && p.arch == arch)
            .expect("a cef pin")
    }

    /// The manifest with one key of the first cef row renamed away, so the row lacks it.
    fn without_cef_key(key: &str) -> String {
        let at = MANIFEST.find("name = \"cef\"").expect("cef rows exist");
        let (head, tail) = MANIFEST.split_at(at);
        let needle = format!("\n{key} = ");
        assert!(tail.contains(&needle), "the cef row has {key}");
        format!("{head}{}", tail.replacen(&needle, &format!("\nx_{key} = "), 1))
    }

    #[test]
    fn the_committed_manifest_pins_cef_for_three_architectures() {
        let all = parse_manifest(MANIFEST).expect("manifest parses");
        let pins: Vec<&Pin> = all.iter().filter(|p| p.is_cef()).collect();
        let arches: Vec<&str> = pins.iter().map(|p| p.arch.as_str()).collect();
        assert_eq!(arches, ["x86_64", "i686", "aarch64"]);
        for pin in &pins {
            assert!(
                pin.url
                    .starts_with("https://cef-builds.spotifycdn.com/cef_binary_156.0.31%2B")
            );
            assert!(pin.url.ends_with("_minimal.tar.bz2"));
            assert_eq!(
                pin.version.as_deref(),
                Some("156.0.31+ge603d97+chromium-156.0.8078.100")
            );
            assert_eq!(pin.layout.as_deref(), Some("tree"));
            assert_eq!(pin.license, "BSD-3-Clause");
            assert_eq!(pin.cef_version_folder(), "156.0.31");
        }
        assert_eq!(
            pins[0].upstream_sha1.as_deref(),
            Some("d179fb251141d6c30992a1775db80a6a879852ba")
        );
        assert_eq!(all.iter().filter(|p| p.name == "libmpv").count(), 3);
    }

    #[test]
    fn a_cef_row_without_its_required_fields_is_refused() {
        for key in ["sha256", "version", "upstream_sha1", "layout", "size"] {
            assert!(parse_manifest(&without_cef_key(key)).is_err(), "{key}");
        }
        assert!(parse_manifest(MANIFEST).is_ok());
    }

    #[test]
    fn a_second_cef_row_for_one_architecture_or_a_plain_http_url_is_refused() {
        let at = MANIFEST.find("name = \"cef\"").expect("cef rows exist");
        let (head, tail) = MANIFEST.split_at(at);
        let duplicate = format!("{head}{}", tail.replacen("arch = \"i686\"", "arch = \"x86_64\"", 1));
        assert!(parse_manifest(&duplicate).is_err());
        let plain_http = format!("{head}{}", tail.replacen("https://", "http://", 1));
        assert!(parse_manifest(&plain_http).is_err());
        let bad_sha1 = format!(
            "{head}{}",
            tail.replacen("d179fb251141d6c30992a1775db80a6a879852ba", "d179", 1)
        );
        assert!(parse_manifest(&bad_sha1).is_err());
        let wrong_layout = format!("{head}{}", tail.replacen("layout = \"tree\"", "layout = \"flat\"", 1));
        assert!(parse_manifest(&wrong_layout).is_err());
        // One libmpv and one cef row for the same architecture are fine.
        assert!(parse_manifest(MANIFEST).is_ok());
    }

    #[test]
    fn the_cef_layout_is_the_folder_cef_dll_sys_reads() {
        let target = Path::new("target");
        let x64 = cef_pin("x86_64");
        let dist = cef_dist_dir(&x64, target);
        assert!(
            dist.ends_with(Path::new("engines/cef/156.0.31/cef_windows_x86_64")),
            "{dist:?}"
        );
        assert!(cef_dist_dir(&cef_pin("i686"), target).ends_with("cef_windows_x86"));
        assert!(cef_dist_dir(&cef_pin("aarch64"), target).ends_with("cef_windows_aarch64"));
        assert_eq!(
            x64.cef_archive_name(),
            "cef_binary_156.0.31+ge603d97+chromium-156.0.8078.100_windows64_minimal.tar.bz2"
        );
        assert_eq!(
            x64.cef_top_folder(),
            "cef_binary_156.0.31+ge603d97+chromium-156.0.8078.100_windows64_minimal"
        );
        let archive = archive_path(&x64, target);
        assert!(
            archive.starts_with(Path::new("target/engines/cef/downloads")),
            "{archive:?}"
        );
        assert!(!archive.starts_with(dist.parent().expect("version folder")));
    }

    /// One tar entry written from raw header bytes, so unsafe names and types can be built.
    fn raw_entry(builder: &mut tar::Builder<Vec<u8>>, name: &str, kind: tar::EntryType, data: &[u8]) {
        let mut header = tar::Header::new_gnu();
        header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
        header.set_entry_type(kind);
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, data).expect("append entry");
    }

    /// A tar.bz2 named like the pin's archive, holding the given (name, type, bytes) entries.
    fn cef_archive(folder: &Path, pin: &Pin, entries: &[(&str, tar::EntryType, &[u8])]) -> PathBuf {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, kind, data) in entries {
            raw_entry(&mut builder, name, *kind, data);
        }
        let tarball = builder.into_inner().expect("finish tar");
        let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
        encoder.write_all(&tarball).expect("compress");
        let path = folder.join(pin.cef_archive_name());
        fs::write(&path, encoder.finish().expect("finish bz2")).expect("write archive");
        path
    }

    fn good_entries(top: &str) -> Vec<(String, tar::EntryType, Vec<u8>)> {
        let file = tar::EntryType::Regular;
        vec![
            (format!("{top}/"), tar::EntryType::Directory, Vec::new()),
            (format!("{top}/Release/libcef.dll"), file, b"dll".to_vec()),
            (format!("{top}/Release/libcef.lib"), file, b"lib".to_vec()),
            (format!("{top}/Resources/icudtl.dat"), file, b"icu".to_vec()),
            (format!("{top}/Resources/locales/en-US.pak"), file, b"pak".to_vec()),
            (format!("{top}/CMakeLists.txt"), file, b"cmake".to_vec()),
            (format!("{top}/include/cef_api_versions.h"), file, b"header".to_vec()),
            (format!("{top}/libcef_dll/wrapper.cc"), file, b"wrapper".to_vec()),
            (format!("{top}/README.txt"), file, b"readme".to_vec()),
            (format!("{top}/tests/ceftests.exe"), file, b"dropped".to_vec()),
        ]
    }

    fn borrowed(entries: &[(String, tar::EntryType, Vec<u8>)]) -> Vec<(&str, tar::EntryType, &[u8])> {
        entries.iter().map(|(n, k, d)| (n.as_str(), *k, d.as_slice())).collect()
    }

    #[test]
    fn cef_extraction_builds_the_layout_download_cef_makes() {
        let scratch = Scratch::new("cef-layout");
        let pin = cef_pin("x86_64");
        let entries = good_entries(&pin.cef_top_folder());
        let archive = cef_archive(&scratch.0, &pin, &borrowed(&entries));
        let dist = scratch.0.join("cef").join("156.0.31").join("cef_windows_x86_64");

        install_cef(&archive, &pin, &dist).expect("extracts");
        assert_eq!(fs::read(dist.join("libcef.dll")).expect("dll"), b"dll");
        assert_eq!(fs::read(dist.join("libcef.lib")).expect("lib"), b"lib");
        assert_eq!(fs::read(dist.join("icudtl.dat")).expect("icu"), b"icu");
        assert_eq!(fs::read(dist.join("locales").join("en-US.pak")).expect("pak"), b"pak");
        assert_eq!(fs::read(dist.join("CMakeLists.txt")).expect("cmake"), b"cmake");
        assert_eq!(
            fs::read(dist.join("include").join("cef_api_versions.h")).expect("header"),
            b"header"
        );
        assert_eq!(
            fs::read(dist.join("libcef_dll").join("wrapper.cc")).expect("wrapper"),
            b"wrapper"
        );
        assert!(!dist.join("Release").exists() && !dist.join("Resources").exists());
        assert!(
            !dist.join("tests").exists(),
            "entries the build never reads are dropped"
        );
        assert!(!dist.with_file_name("extracting_cef_windows_x86_64").exists());

        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dist.join("archive.json")).expect("archive.json")).expect("json");
        assert_eq!(json["type"], "minimal");
        assert_eq!(json["name"], pin.cef_archive_name());
        assert_eq!(json["sha1"], text(&pin.upstream_sha1));
    }

    #[test]
    fn cef_extraction_refuses_unsafe_entries_and_writes_nothing() {
        let pin = cef_pin("x86_64");
        let top = pin.cef_top_folder();
        let cases: Vec<(&str, String, tar::EntryType)> = vec![
            ("absolute", "/etc/passwd".to_owned(), tar::EntryType::Regular),
            ("parent", format!("{top}/../escape.txt"), tar::EntryType::Regular),
            ("drive", format!("{top}/Release/C:evil.dll"), tar::EntryType::Regular),
            (
                "backslash",
                format!("{top}/Release/a\\..\\b.dll"),
                tar::EntryType::Regular,
            ),
            ("symlink", format!("{top}/Release/link"), tar::EntryType::Symlink),
            ("hardlink", format!("{top}/Release/hard"), tar::EntryType::Link),
            ("device", format!("{top}/Release/dev"), tar::EntryType::Char),
            ("block device", format!("{top}/Release/blk"), tar::EntryType::Block),
            ("fifo", format!("{top}/Release/fifo"), tar::EntryType::Fifo),
            (
                "other top folder",
                "other_top/Release/libcef.dll".to_owned(),
                tar::EntryType::Regular,
            ),
            (
                "link outside the used folders",
                format!("{top}/tests/link"),
                tar::EntryType::Symlink,
            ),
        ];
        for (label, name, kind) in cases {
            let scratch = Scratch::new(&format!("cef-unsafe-{}", label.replace(' ', "-")));
            let mut entries = good_entries(&top);
            entries.insert(3, (name, kind, Vec::new()));
            let archive = cef_archive(&scratch.0, &pin, &borrowed(&entries));
            let dist = scratch.0.join("cef").join("156.0.31").join("cef_windows_x86_64");

            let error = install_cef(&archive, &pin, &dist).expect_err(label);
            assert!(!error.is_empty(), "{label}");
            assert!(!dist.exists(), "{label}: the dist must not exist");
            assert!(
                !dist.with_file_name("extracting_cef_windows_x86_64").exists(),
                "{label}: no staging folder is left"
            );
            assert!(!scratch.0.join("escape.txt").exists(), "{label}");
        }
    }

    #[test]
    fn cef_extraction_refuses_an_archive_without_the_runtime_files() {
        let scratch = Scratch::new("cef-incomplete");
        let pin = cef_pin("x86_64");
        let top = pin.cef_top_folder();
        let archive = cef_archive(
            &scratch.0,
            &pin,
            &[(&format!("{top}/CMakeLists.txt"), tar::EntryType::Regular, b"cmake")],
        );
        let dist = scratch.0.join("cef").join("156.0.31").join("cef_windows_x86_64");
        let error = install_cef(&archive, &pin, &dist).expect_err("no libcef.dll inside");
        assert!(error.contains("libcef.dll"), "{error}");
        assert!(!dist.exists());
    }

    #[test]
    fn ci_knows_whether_the_cef_dist_of_the_host_is_fetched() {
        let scratch = Scratch::new("cef-fetched");
        assert!(!dist_fetched(&scratch.0, "x86_64"));
        assert!(!dist_fetched(&scratch.0, "aarch64"));
        assert!(dist_fetched(&scratch.0, "i686"), "x86 builds no CEF");
        let dist = cef_dist_dir(&cef_pin("x86_64"), &scratch.0);
        fs::create_dir_all(&dist).expect("create");
        assert!(
            !dist_fetched(&scratch.0, "x86_64"),
            "a folder without archive.json is not a dist"
        );
        fs::write(dist.join("archive.json"), b"{}").expect("write");
        assert!(dist_fetched(&scratch.0, "x86_64"));
        assert!(!dist_fetched(&scratch.0, "aarch64"));
    }

    #[test]
    fn cef_extraction_replaces_a_stale_dist() {
        let scratch = Scratch::new("cef-replace");
        let pin = cef_pin("x86_64");
        let entries = good_entries(&pin.cef_top_folder());
        let archive = cef_archive(&scratch.0, &pin, &borrowed(&entries));
        let dist = scratch.0.join("cef").join("156.0.31").join("cef_windows_x86_64");
        fs::create_dir_all(&dist).expect("create");
        fs::write(dist.join("stale.txt"), b"old").expect("write");
        install_cef(&archive, &pin, &dist).expect("extracts");
        assert!(!dist.join("stale.txt").exists());
        assert!(dist.join("libcef.dll").is_file());
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
