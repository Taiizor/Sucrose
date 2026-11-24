//! `cargo xtask synth-library`: a synthetic library and prepared settings for UI runs and the D-25 measurement.
//!
//! `--out <dir>` gets `Library/` with `--count` valid themes (real-sized JPEG thumbnails and GIF previews), two corrupt
//! folders and a loose file, plus `Setting/Sucrose.json`: its library location is that library, nothing is selected, the
//! tray is hidden and the tray hint is off. Started with `SUCROSE_DATA_DIR=<dir>` the service loads the file as it is
//! and plays nothing, so no test-only app flag is needed (D-25). Only `<dir>` is written; the real data folder is
//! refused as `--out`.

use std::{
    collections::BTreeSet,
    fs,
    io::{self},
    path::{Component, Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use image::{
    Delay, ExtendedColorType, Frame, RgbaImage, codecs::gif::GifEncoder, codecs::gif::Repeat, codecs::jpeg::JpegEncoder,
};
use sucrose_config::{
    model::Settings,
    store::{SETTINGS_FILE, write_atomic},
};
use sucrose_core::{
    library, paths,
    theme::{SucroseInfo, WallpaperType},
    version::{LegacyVersion, app_version},
};

/// Result of the generator; the error is the message printed to the user.
type Outcome<T> = Result<T, String>;

/// Marker file that says a folder is synthetic output of this tool (count and seed).
const MARKER: &str = ".synth";
/// Folder with the pool of generated images the theme folders copy from.
const POOL: &str = ".pool";
/// Largest library the tool writes.
const MAX_COUNT: usize = 1000;
/// Corrupt folders added after the themes.
const CORRUPT: usize = 2;
/// App version of an ordinary synthetic theme.
const APP_VERSION: &str = "26.10.19.0";
/// App version of every 50th theme: newer than this build, so the card shows it as needing a newer app (LIB-08).
const NEWER_APP_VERSION: &str = "99.1.1.0";
/// The "video" of a synthetic theme: 16 bytes, not playable. The measurement never plays it.
const FAKE_VIDEO: &[u8; 16] = b"SYNTHETIC-NOTVID";
/// Characters of a theme folder name (formats.md section 2).
const NAME_CHARS: &[u8; 62] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const NAME_LENGTH: usize = 25;
const DEFAULT_SEED: u64 = 1;

/// What one run of the tool was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Folder that receives `Library/`, `Setting/` and the marker.
    pub out: PathBuf,
    /// Number of valid themes, 1..=1000.
    pub count: usize,
    /// Seed of the folder names and the image content.
    pub seed: u64,
    /// Allows a non-empty `--out` that is not synthetic output.
    pub force: bool,
}

impl Plan {
    /// Reads `--out <dir> --count <n> [--seed <u64>] [--force]`.
    fn parse(args: &[String]) -> Outcome<Self> {
        let mut out = None;
        let mut count = None;
        let mut seed = DEFAULT_SEED;
        let mut force = false;
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            let mut value = |flag: &str| it.next().ok_or_else(|| format!("{flag} needs a value"));
            match arg.as_str() {
                "--out" => out = Some(PathBuf::from(value("--out")?)),
                "--count" => {
                    let text = value("--count")?;
                    count = Some(
                        text.parse::<usize>()
                            .ok()
                            .filter(|count| (1..=MAX_COUNT).contains(count))
                            .ok_or_else(|| format!("--count must be a number from 1 to {MAX_COUNT}, not '{text}'"))?,
                    );
                }
                "--seed" => {
                    let text = value("--seed")?;
                    seed = text
                        .parse::<u64>()
                        .map_err(|_| format!("--seed must be a whole number, not '{text}'"))?;
                }
                "--force" => force = true,
                other => return Err(format!("unknown argument '{other}'")),
            }
        }
        Ok(Self {
            out: out.ok_or("give --out <dir>")?,
            count: count.ok_or("give --count <n>")?,
            seed,
            force,
        })
    }
}

/// `cargo xtask synth-library --out <dir> --count <n> [--seed <u64>] [--force]`
pub fn run(args: &[String]) -> ExitCode {
    let result = Plan::parse(args).and_then(|plan| generate(&plan));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("synth-library: {message}");
            eprintln!("usage: cargo xtask synth-library --out <dir> --count <n> [--seed <u64>] [--force]");
            ExitCode::FAILURE
        }
    }
}

/// A small xorshift generator, so no random-number crate is needed; the output only has to be well spread and the
/// same for the same seed.
struct XorShift(u64);

impl XorShift {
    fn new(state: u64) -> Self {
        Self(if state == 0 { 0x9E37_79B9_7F4A_7C15 } else { state })
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// The splitmix64 finalizer: a cheap, well-spread hash of one number.
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// The folder name of theme `index`: 25 characters of `[A-Za-z0-9]`, the same for the same seed and index.
pub fn theme_name(seed: u64, index: usize) -> String {
    let mut rng = XorShift::new(mix(seed ^ mix(index as u64 + 1)));
    (0..NAME_LENGTH)
        .map(|_| char::from(NAME_CHARS[(rng.next() % NAME_CHARS.len() as u64) as usize]))
        .collect()
}

/// A JPEG thumbnail of the pool: its size and the file size to land near.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThumbSpec {
    width: u32,
    height: u32,
    target_bytes: usize,
}

/// A GIF preview of the pool: its frame size and the file size to land near.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreviewSpec {
    width: u32,
    height: u32,
    target_bytes: usize,
}

/// The images the theme folders copy from.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PoolPlan {
    thumbnails: Vec<ThumbSpec>,
    previews: Vec<PreviewSpec>,
}

/// The encoded files of the pool, in plan order.
struct PoolFiles {
    thumbnails: Vec<Vec<u8>>,
    previews: Vec<Vec<u8>>,
}

/// The pool to generate. The first version is a single small image of each kind; the real-sized pool replaces it.
fn pool_plan() -> PoolPlan {
    PoolPlan {
        thumbnails: vec![ThumbSpec {
            width: 320,
            height: 180,
            target_bytes: 20_000,
        }],
        previews: vec![PreviewSpec {
            width: 64,
            height: 36,
            target_bytes: 20_000,
        }],
    }
}

/// Deterministic picture: a gradient that moves with `frame` plus noise of the given amplitude. The noise keeps the
/// encoded size high, like a photograph, and every frame differs so a GIF cannot collapse into one frame.
fn rgb_buffer(width: u32, height: u32, frame: u32, seed: u64, noise: i32) -> Vec<u8> {
    let (w, h) = (width as usize, height as usize);
    let mut bytes = Vec::with_capacity(w * h * 3);
    let hue = (seed % 97) as usize;
    let key = mix(seed ^ (u64::from(frame) << 40));
    for y in 0..h {
        for x in 0..w {
            let base = [
                40 + (x * 160 / w + hue) % 160,
                30 + y * 180 / h,
                (x + y + usize::try_from(frame).unwrap_or(0) * 8) * 255 / (w + h),
            ];
            let hash = mix(key ^ ((y as u64) << 20) ^ x as u64);
            for (channel, value) in base.into_iter().enumerate() {
                let jitter = ((hash >> (channel * 8)) % (2 * noise as u64 + 1)) as i32 - noise;
                bytes.push((value as i32 + jitter).clamp(0, 255) as u8);
            }
        }
    }
    bytes
}

fn encode_jpeg(rgb: &[u8], width: u32, height: u32, quality: u8) -> Outcome<Vec<u8>> {
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, quality)
        .encode(rgb, width, height, ExtendedColorType::Rgb8)
        .map_err(|error| format!("cannot encode a thumbnail: {error}"))?;
    Ok(bytes)
}

/// A JPEG whose size is as near as possible to the target without passing it: the highest quality that fits.
fn make_thumbnail(spec: &ThumbSpec, seed: u64, index: usize) -> Outcome<Vec<u8>> {
    let rgb = rgb_buffer(spec.width, spec.height, 0, seed ^ index as u64, 14);
    let (mut low, mut high) = (1_u8, 100_u8);
    while low < high {
        let middle = (low + high).div_ceil(2);
        if encode_jpeg(&rgb, spec.width, spec.height, middle)?.len() <= spec.target_bytes {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    encode_jpeg(&rgb, spec.width, spec.height, low)
}

fn encode_gif(width: u32, height: u32, frames: u32, seed: u64, noise: i32) -> Outcome<Vec<u8>> {
    let mut bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new_with_speed(&mut bytes, 30);
        let failed = |error: image::ImageError| format!("cannot encode a preview: {error}");
        encoder.set_repeat(Repeat::Infinite).map_err(failed)?;
        for frame in 0..frames {
            let rgb = rgb_buffer(width, height, frame, seed, noise);
            let rgba: Vec<u8> = rgb
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect();
            let image = RgbaImage::from_raw(width, height, rgba).ok_or("a preview frame has the wrong size")?;
            encoder
                .encode_frame(Frame::from_parts(image, 0, 0, Delay::from_numer_denom_ms(100, 1)))
                .map_err(failed)?;
        }
    }
    Ok(bytes)
}

/// How many frames of the size of the first one reach the target (at least one).
fn frames_for(target_bytes: usize, first_frame_bytes: usize) -> u32 {
    let frames = target_bytes.div_ceil(first_frame_bytes.max(1));
    u32::try_from(frames.max(1)).unwrap_or(u32::MAX)
}

/// A GIF with as many frames as bring the file near the target size.
fn make_preview(spec: &PreviewSpec, seed: u64, index: usize) -> Outcome<Vec<u8>> {
    let seed = seed ^ (index as u64).wrapping_mul(0x517C_C1B7);
    let first = encode_gif(spec.width, spec.height, 1, seed, 30)?;
    let frames = frames_for(spec.target_bytes, first.len());
    if frames == 1 {
        return Ok(first);
    }
    encode_gif(spec.width, spec.height, frames, seed, 30)
}

fn io_error(action: &str, path: &Path) -> impl FnOnce(io::Error) -> String {
    let (action, path) = (action.to_owned(), path.display().to_string());
    move |error| format!("cannot {action} {path}: {error}")
}

fn write_file(path: &Path, bytes: &[u8]) -> Outcome<()> {
    fs::write(path, bytes).map_err(io_error("write", path))
}

fn create_dir(path: &Path) -> Outcome<()> {
    fs::create_dir_all(path).map_err(io_error("create", path))
}

/// Removes a folder this tool owns; a missing one is fine.
fn remove_dir(path: &Path) -> Outcome<()> {
    match fs::remove_dir_all(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(io_error("remove", path)(error)),
        _ => Ok(()),
    }
}

/// The path without `.` and `..` components, lowercase, as a list of names; two spellings of one Windows folder
/// compare equal.
fn lower_components(path: &Path) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            other => parts.push(other.as_os_str().to_string_lossy().to_lowercase()),
        }
    }
    parts
}

/// Whether `path` is `root` or inside it.
fn is_inside(path: &Path, root: &Path) -> bool {
    lower_components(path).starts_with(&lower_components(root))
}

/// The text of the marker of a finished run.
fn marker_text(count: usize, seed: u64) -> String {
    format!("count={count}\nseed={seed}\n")
}

/// The theme description of theme `index`.
fn theme_info(index: usize, kind: WallpaperType, source: &str, app_version: &str) -> Outcome<SucroseInfo> {
    let version = |text: &str| LegacyVersion::parse(text).map_err(|error| format!("bad version {text}: {error}"));
    Ok(SucroseInfo {
        app_version: version(app_version)?,
        version: version("1.0.0.0")?,
        title: format!("Synthetic theme {index}"),
        thumbnail: "Thumbnail.jpg".to_owned(),
        preview: "Preview.gif".to_owned(),
        description: format!("Synthetic theme {index}, made by cargo xtask synth-library for UI and measurement runs."),
        author: "Sucrose".to_owned(),
        license: None,
        contact: "https://sucrose.dev".to_owned(),
        kind,
        source: source.to_owned(),
        tags: Some(vec!["synthetic".to_owned(), "benchmark".to_owned()]),
        arguments: None,
        localization: None,
        extra: serde_json::Map::new(),
    })
}

fn info_bytes(info: &SucroseInfo) -> Outcome<Vec<u8>> {
    serde_json::to_vec_pretty(info).map_err(|error| format!("cannot write a SucroseInfo.json: {error}"))
}

/// Writes the pool and returns the file contents in plan order.
fn write_pool(pool_dir: &Path, plan: &Plan, pool: &PoolPlan) -> Outcome<PoolFiles> {
    create_dir(pool_dir)?;
    let mut thumbnails = Vec::new();
    for (index, spec) in pool.thumbnails.iter().enumerate() {
        let bytes = make_thumbnail(spec, plan.seed, index)?;
        write_file(&pool_dir.join(format!("thumbnail-{index:02}.jpg")), &bytes)?;
        thumbnails.push(bytes);
    }
    let mut previews = Vec::new();
    for (index, spec) in pool.previews.iter().enumerate() {
        let bytes = make_preview(spec, plan.seed, index)?;
        write_file(&pool_dir.join(format!("preview-{index:02}.gif")), &bytes)?;
        previews.push(bytes);
    }
    Ok(PoolFiles { thumbnails, previews })
}

/// Writes the whole synthetic folder, checks it with the library scan and prints the result.
fn generate(plan: &Plan) -> Outcome<()> {
    let started = Instant::now();
    let out = std::path::absolute(&plan.out).map_err(io_error("resolve", &plan.out))?;
    if let Some(real) = paths::legacy_data_dir()
        && is_inside(&out, &real)
    {
        return Err(format!(
            "{} is inside the real Sucrose data folder; the generator never writes there",
            out.display()
        ));
    }
    if out.exists() {
        let marked = out.join(MARKER).is_file();
        let empty = fs::read_dir(&out).map_err(io_error("read", &out))?.next().is_none();
        if !marked && !empty && !plan.force {
            return Err(format!(
                "{} is not empty and not synthetic output; use --force to write into it anyway",
                out.display()
            ));
        }
    }
    let library_dir = out.join("Library");
    let setting_dir = out.join("Setting");
    let pool_dir = out.join(POOL);
    create_dir(&out)?;
    // The marker goes first (unfinished), so a run that dies halfway can be repeated without --force.
    write_file(&out.join(MARKER), b"incomplete\n")?;
    for dir in [&library_dir, &setting_dir, &pool_dir] {
        remove_dir(dir)?;
    }
    create_dir(&library_dir)?;

    let PoolFiles { thumbnails, previews } = write_pool(&pool_dir, plan, &pool_plan())?;
    let mut names = BTreeSet::new();
    for index in 0..plan.count + CORRUPT {
        if !names.insert(theme_name(plan.seed, index)) {
            return Err("two themes got the same folder name; use another --seed".to_owned());
        }
    }
    let placeholder_gif = encode_gif(32, 18, 2, plan.seed, 20)?;
    for index in 0..plan.count {
        let dir = library_dir.join(theme_name(plan.seed, index));
        create_dir(&dir)?;
        let (kind, source) = match (index % 4, (index / 4) % 2) {
            (3, 0) => (WallpaperType::Web, "index.html"),
            (3, _) => (WallpaperType::Gif, "Animation.gif"),
            _ => (WallpaperType::Video, "Video.mp4"),
        };
        let app = if (index + 1) % 50 == 0 {
            NEWER_APP_VERSION
        } else {
            APP_VERSION
        };
        write_file(
            &dir.join("SucroseInfo.json"),
            &info_bytes(&theme_info(index, kind, source, app)?)?,
        )?;
        write_file(&dir.join("Thumbnail.jpg"), &thumbnails[index % thumbnails.len()])?;
        write_file(&dir.join("Preview.gif"), &previews[index % previews.len()])?;
        match kind {
            WallpaperType::Web => write_file(&dir.join(source), b"<!doctype html><title>Synthetic</title>\n")?,
            WallpaperType::Gif => write_file(&dir.join(source), &placeholder_gif)?,
            _ => write_file(&dir.join(source), FAKE_VIDEO)?,
        }
    }

    // Corrupt folder one: valid description, thumbnail and source, but no Preview.gif.
    let no_preview = library_dir.join(theme_name(plan.seed, plan.count));
    create_dir(&no_preview)?;
    let info = theme_info(plan.count, WallpaperType::Video, "Video.mp4", APP_VERSION)?;
    write_file(&no_preview.join("SucroseInfo.json"), &info_bytes(&info)?)?;
    write_file(&no_preview.join("Thumbnail.jpg"), &thumbnails[0])?;
    write_file(&no_preview.join("Video.mp4"), FAKE_VIDEO)?;
    // Corrupt folder two: SucroseInfo.json is not JSON.
    let broken = library_dir.join(theme_name(plan.seed, plan.count + 1));
    create_dir(&broken)?;
    write_file(&broken.join("SucroseInfo.json"), b"{ \"Title\": \"broken\", ")?;
    write_file(
        &library_dir.join("notes.txt"),
        b"A loose file: the library scan lists folders only.\n",
    )?;

    create_dir(&setting_dir)?;
    let mut settings = Settings::default();
    settings.library.location = library_dir.to_string_lossy().into_owned();
    settings.library.selected = String::new();
    settings.library.delete_corrupt = false;
    settings.general.app_visible = false;
    settings.warehouse.hint_tray_icon = false;
    let bytes = settings
        .to_bytes()
        .map_err(|error| format!("cannot write the settings: {error}"))?;
    write_atomic(&setting_dir.join(SETTINGS_FILE), &bytes).map_err(io_error("write", &setting_dir))?;

    let scan = library::scan(&library_dir, &app_version()).map_err(io_error("scan", &library_dir))?;
    if scan.themes.len() != plan.count || scan.corrupt.len() != CORRUPT {
        return Err(format!(
            "the library scan found {} themes and {} corrupt folders, expected {} and {CORRUPT}",
            scan.themes.len(),
            scan.corrupt.len(),
            plan.count
        ));
    }
    write_file(&out.join(MARKER), marker_text(plan.count, plan.seed).as_bytes())?;
    println!(
        "synth library: {} themes, {} corrupt, {} ignored",
        scan.themes.len(),
        scan.corrupt.len(),
        scan.ignored
    );
    println!(
        "  {} in {:.1} s at {}",
        size_text(folder_size(&out)),
        started.elapsed().as_secs_f64(),
        out.display()
    );
    Ok(())
}

/// Total size of the files under `dir`.
fn folder_size(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.metadata() {
            Ok(metadata) if metadata.is_dir() => folder_size(&entry.path()),
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        })
        .sum()
}

fn size_text(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
}
