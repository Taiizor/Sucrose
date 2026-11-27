//! `cargo xtask synth-library`: a synthetic library and prepared settings for UI runs and the D-25 measurement.
//!
//! `--out <dir>` gets `Library/` with `--count` valid themes (real-sized JPEG thumbnails and GIF previews), two corrupt
//! folders and a loose file, plus `Setting/Sucrose.json`: its library location is that library, nothing is selected, the
//! tray is hidden and the tray hint is off. Started with `SUCROSE_DATA_DIR=<dir>` the service loads the file as it is
//! and plays nothing, so no test-only app flag is needed (D-25). Only `<dir>` is written; `--out` is refused when it is,
//! holds or lies inside the real Sucrose data folder or the data folder in effect (`SUCROSE_DATA_DIR`), however the
//! path is spelled (links, junctions, 8.3 names and the verbatim prefix are resolved first); `--force` never bypasses
//! it, so run the tool without `SUCROSE_DATA_DIR` set.
//!
//! `--web-fixtures` also copies the fixture Web themes of `apps/sucrose/tests/fixtures/web-themes` into the library
//! (a Thumbnail and a Preview they do not carry are written from the pool), and `--select <folder>` selects one of the
//! themes the run wrote, so an engine run needs neither the maintainer's library nor his settings. `--media-fixtures`
//! adds the animation themes `gif-anim`, `apng-anim` and `webp-anim` (480 x 270, 24 frames of 50 ms that draw their frame
//! number as blocks, built by `media.rs`) and the Video theme `video-clip` (placeholder clip bytes), which `--select`
//! can name too. `--engine
//! <Type>=<EngineType>` (repeatable, legacy names) sets the engine of a wallpaper type in those settings; an engine the
//! type does not allow, or an unknown type, is refused before anything is written.

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

use crate::media;
use sucrose_config::{
    model::Settings,
    store::{SETTINGS_FILE, write_atomic},
};
use sucrose_core::{
    library, paths,
    settings::EngineType,
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
    /// Also copies the fixture Web themes into the library.
    pub web_fixtures: bool,
    /// Also writes the media fixture themes (`gif-anim`, `apng-anim`, `webp-anim`, `video-clip`) into the library.
    pub media_fixtures: bool,
    /// Folder name of a theme this run writes, selected in the prepared settings.
    pub select: Option<String>,
    /// The engine of a wallpaper type in the prepared settings (`--engine <Type>=<EngineType>`), in the order given.
    pub engines: Vec<(WallpaperType, EngineType)>,
}

/// The engines the legacy app allows for a wallpaper type.
fn allowed_engines(kind: WallpaperType) -> &'static [EngineType] {
    match kind {
        WallpaperType::Gif => EngineType::GIF,
        WallpaperType::Url => EngineType::URL,
        WallpaperType::Web => EngineType::WEB,
        WallpaperType::Video => EngineType::VIDEO,
        WallpaperType::YouTube => EngineType::YOUTUBE,
        WallpaperType::Application => EngineType::APPLICATION,
    }
}

/// Reads one `--engine` value, `<Type>=<EngineType>`, both legacy names: the engine must be one the type allows.
fn parse_engine(text: &str) -> Outcome<(WallpaperType, EngineType)> {
    let (kind_text, engine_text) = text
        .split_once('=')
        .ok_or_else(|| format!("--engine must look like <Type>=<EngineType>, not '{text}'"))?;
    let kind = WallpaperType::from_legacy_name(kind_text).ok_or_else(|| {
        format!("--engine {text}: '{kind_text}' is not a wallpaper type (Gif, Url, Web, Video, YouTube, Application)")
    })?;
    let engine = EngineType::from_legacy_name(engine_text)
        .ok_or_else(|| format!("--engine {text}: '{engine_text}' is not an engine name"))?;
    if !allowed_engines(kind).contains(&engine) {
        let allowed: Vec<&str> = allowed_engines(kind)
            .iter()
            .map(|allowed| allowed.legacy_name())
            .collect();
        return Err(format!(
            "--engine {text}: {kind_text} wallpapers allow {} only",
            allowed.join(", ")
        ));
    }
    Ok((kind, engine))
}

/// Sets the engine of `kind` in `settings`.
fn set_engine(settings: &mut Settings, kind: WallpaperType, engine: EngineType) {
    let engines = &mut settings.engine;
    match kind {
        WallpaperType::Gif => engines.gif = engine,
        WallpaperType::Url => engines.url = engine,
        WallpaperType::Web => engines.web = engine,
        WallpaperType::Video => engines.video = engine,
        WallpaperType::YouTube => engines.youtube = engine,
        WallpaperType::Application => engines.application = engine,
    }
}

impl Plan {
    /// Reads `--out <dir> --count <n> [--seed <u64>] [--force] [--web-fixtures] [--media-fixtures] [--select <folder>]
    /// [--engine <Type>=<EngineType>]...`.
    fn parse(args: &[String]) -> Outcome<Self> {
        let mut out = None;
        let mut count = None;
        let mut seed = DEFAULT_SEED;
        let mut force = false;
        let mut web_fixtures = false;
        let mut media_fixtures = false;
        let mut select = None;
        let mut engines = Vec::new();
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
                "--web-fixtures" => web_fixtures = true,
                "--media-fixtures" => media_fixtures = true,
                "--select" => select = Some(value("--select")?.clone()),
                "--engine" => engines.push(parse_engine(value("--engine")?)?),
                other => return Err(format!("unknown argument '{other}'")),
            }
        }
        Ok(Self {
            out: out.ok_or("give --out <dir>")?,
            count: count.ok_or("give --count <n>")?,
            seed,
            force,
            web_fixtures,
            media_fixtures,
            select,
            engines,
        })
    }
}

/// `cargo xtask synth-library --out <dir> --count <n> [--seed <u64>] [--force] [--web-fixtures] [--media-fixtures]
/// [--select <folder>] [--engine <Type>=<EngineType>]...`
pub fn run(args: &[String]) -> ExitCode {
    let result = Plan::parse(args).and_then(|plan| generate(&plan));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("synth-library: {message}");
            eprintln!(
                "usage: cargo xtask synth-library --out <dir> --count <n> [--seed <u64>] [--force] [--web-fixtures] \
                 [--media-fixtures] [--select <folder>] [--engine <Type>=<EngineType>]..."
            );
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

/// The pool to generate, shaped like the maintainer's library (finding F1): 12 JPEG thumbnails of 1280x720 from about
/// 100 KB to 640 KB with a median near 190 KB, and 8 GIF previews from about 0.2 MB to 4.9 MB with a median near
/// 1.5 MB. The theme folders copy from it round-robin, so a 500-theme library still has varied sizes.
fn pool_plan() -> PoolPlan {
    const THUMBNAIL_BYTES: [usize; 12] = [
        100_000, 125_000, 150_000, 170_000, 185_000, 190_000, 200_000, 220_000, 260_000, 330_000, 450_000, 640_000,
    ];
    // Frame size by file size: small previews use small frames so one frame never overshoots a small target.
    const PREVIEWS: [(u32, u32, usize); 8] = [
        (320, 180, 200_000),
        (480, 270, 700_000),
        (640, 360, 1_100_000),
        (640, 360, 1_300_000),
        (640, 360, 1_600_000),
        (640, 360, 2_400_000),
        (640, 360, 4_200_000),
        (640, 360, 4_900_000),
    ];
    PoolPlan {
        thumbnails: THUMBNAIL_BYTES
            .into_iter()
            .map(|target_bytes| ThumbSpec {
                width: 1280,
                height: 720,
                target_bytes,
            })
            .collect(),
        previews: PREVIEWS
            .into_iter()
            .map(|(width, height, target_bytes)| PreviewSpec {
                width,
                height,
                target_bytes,
            })
            .collect(),
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

/// Whether `path` is `root` or inside it (lexically).
fn is_inside(path: &Path, root: &Path) -> bool {
    lower_components(path).starts_with(&lower_components(root))
}

/// The path without the Windows verbatim prefix: `\\?\UNC\server\share` becomes `\\server\share` and `\\?\C:\x`
/// becomes `C:\x`, so two spellings of one folder compare equal.
fn strip_verbatim(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else if let Some(rest) = text.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path.to_path_buf()
    }
}

/// The real location of `path` as far as it exists: every component that exists is resolved through links, junctions
/// and 8.3 names (`canonicalize`, one step at a time so a `..` after a link goes where the OS goes; Windows removes
/// the `..` in `std::path::absolute` first, as Win32 does), the rest, which
/// does not exist yet, is appended as written. The verbatim prefix is stripped.
fn resolve(path: &Path) -> PathBuf {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut resolved = PathBuf::new();
    let mut existing = true;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => resolved.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if existing && let Ok(real) = fs::canonicalize(resolved.join("..")) {
                    resolved = real;
                } else {
                    existing = false;
                    resolved.pop();
                }
            }
            Component::Normal(name) => {
                let next = resolved.join(name);
                match fs::canonicalize(&next) {
                    Ok(real) if existing => resolved = real,
                    _ => {
                        existing = false;
                        resolved = next;
                    }
                }
            }
        }
    }
    strip_verbatim(&resolved)
}

/// Whether `out` is, holds or lies inside one of the `guarded` folders, whatever the spelling: both sides are resolved
/// first (links, junctions, 8.3 names, the verbatim prefix, `.` and `..`), then compared by components without regard to
/// case.
fn refuses(out: &Path, guarded: &[PathBuf]) -> bool {
    let out = resolve(out);
    guarded.iter().any(|root| {
        let root = resolve(root);
        is_inside(&out, &root) || is_inside(&root, &out)
    })
}

/// The folders the generator never writes in, with what to call them in a message: the C# install's data folder and
/// the data folder in effect (`SUCROSE_DATA_DIR`, when set).
fn guarded_folders() -> Vec<(&'static str, PathBuf)> {
    let mut folders = Vec::new();
    if let Some(real) = paths::legacy_data_dir() {
        folders.push(("the real Sucrose data folder", real));
    }
    if let Some(current) = paths::data_dir()
        && !folders.iter().any(|(_, real)| resolve(real) == resolve(&current))
    {
        folders.push(("the data folder in effect (SUCROSE_DATA_DIR)", current));
    }
    folders
}
/// The text of the marker of a finished run.
fn marker_text(count: usize, seed: u64) -> String {
    format!("count={count}\nseed={seed}\n")
}

/// The marker of a finished run of this plan: the count and seed, and the selected theme when there is one.
fn marker_for(plan: &Plan) -> String {
    let mut text = marker_text(plan.count, plan.seed);
    if let Some(select) = &plan.select {
        text.push_str(&format!("select={select}\n"));
    }
    if plan.media_fixtures {
        text.push_str("media-fixtures\n");
    }
    for (kind, engine) in &plan.engines {
        text.push_str(&format!("engine={kind:?}={}\n", engine.legacy_name()));
    }
    text
}

/// The folder and Source file of the GIF animation fixture theme (`--media-fixtures`).
const GIF_FIXTURE_FOLDER: &str = "gif-anim";
const GIF_FIXTURE_SOURCE: &str = "animation.gif";
/// The APNG and WebP animation themes and the video theme `--media-fixtures` adds next to `gif-anim`.
const APNG_FIXTURE_FOLDER: &str = "apng-anim";
const APNG_FIXTURE_SOURCE: &str = "animation.png";
const WEBP_FIXTURE_FOLDER: &str = "webp-anim";
const WEBP_FIXTURE_SOURCE: &str = "animation.webp";
/// A Video theme whose clip is the synthetic placeholder bytes: 04-33 plays it as a generated page, a checkpoint copies
/// a real clip over it.
const VIDEO_FIXTURE_FOLDER: &str = "video-clip";
const VIDEO_FIXTURE_SOURCE: &str = "clip.mp4";
/// The themes `--media-fixtures` writes, by folder name.
const MEDIA_FIXTURE_FOLDERS: [&str; 4] = [
    GIF_FIXTURE_FOLDER,
    APNG_FIXTURE_FOLDER,
    WEBP_FIXTURE_FOLDER,
    VIDEO_FIXTURE_FOLDER,
];

/// The animation fixture: 480 x 270, 24 frames of 50 ms (a 1.2 s loop), the frames of `media::counter_frame`. Nothing
/// random: the same bytes on every run.
fn counter_parts() -> Vec<media::Part> {
    media::counter_parts(480, 270, 24, 50)
}

/// The GIF of the counter animation.
fn encode_counter_gif() -> Outcome<Vec<u8>> {
    let parts = counter_parts();
    let frames: Vec<RgbaImage> = parts.iter().map(|part| part.image.clone()).collect();
    let delays: Vec<u32> = parts.iter().map(|part| part.delay_ms).collect();
    media::animated_gif(&frames, &delays)
}

/// The fixture Web themes (`--web-fixtures`), resolved from the workspace root and never from the current directory.
fn web_fixtures_dir() -> PathBuf {
    crate::workspace_root().join("apps/sucrose/tests/fixtures/web-themes")
}

/// The folder names under the fixtures folder, sorted. A missing or empty folder is an error: `--web-fixtures` was asked
/// for, so a run that silently copies nothing would hide a wrong checkout.
fn fixture_folders(dir: &Path) -> Outcome<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).map_err(io_error("read", dir))? {
        let entry = entry.map_err(io_error("read", dir))?;
        if entry.file_type().map_err(io_error("read", dir))?.is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    if names.is_empty() {
        return Err(format!("no fixture theme folders under {}", dir.display()));
    }
    Ok(names)
}

/// Copies a folder tree of plain files and folders; a link is refused.
fn copy_tree(from: &Path, to: &Path) -> Outcome<()> {
    create_dir(to)?;
    for entry in fs::read_dir(from).map_err(io_error("read", from))? {
        let entry = entry.map_err(io_error("read", from))?;
        let kind = entry.file_type().map_err(io_error("read", from))?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target).map_err(io_error("copy", &entry.path()))?;
        } else {
            return Err(format!(
                "{} is a link; fixtures hold plain files only",
                entry.path().display()
            ));
        }
    }
    Ok(())
}

/// Writes the Thumbnail and Preview a fixture theme names in its `SucroseInfo.json` but does not carry, from the pool,
/// so the library scan accepts it and the repository holds no binary fixture. A name that is not a plain file name is
/// left alone: the folder then shows up as corrupt, which is what the fixture says.
fn fill_missing_images(dir: &Path, thumbnail: &[u8], preview: &[u8]) -> Outcome<()> {
    let path = dir.join("SucroseInfo.json");
    let text = fs::read_to_string(&path).map_err(io_error("read", &path))?;
    let info: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    for (key, bytes) in [("Thumbnail", thumbnail), ("Preview", preview)] {
        let Some(name) = info.get(key).and_then(serde_json::Value::as_str) else {
            continue;
        };
        let plain = Path::new(name).components().count() == 1
            && matches!(Path::new(name).components().next(), Some(Component::Normal(_)));
        if plain && !dir.join(name).exists() {
            write_file(&dir.join(name), bytes)?;
        }
    }
    Ok(())
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

/// What a run did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Generated {
    /// The folder was written.
    Written,
    /// The folder already holds this count and seed and was left as it is.
    UpToDate,
}

/// Whether the text of a marker is that of a finished run of this plan: same count and seed, same selection.
fn marker_matches(text: &str, plan: &Plan) -> bool {
    text == marker_for(plan)
}

/// Whether `out` already holds a finished run of this plan whose library and settings are still there. A run with
/// `--web-fixtures` is never up to date: the fixture files may have changed since.
fn is_up_to_date(out: &Path, plan: &Plan) -> bool {
    let marked =
        !plan.web_fixtures && fs::read_to_string(out.join(MARKER)).is_ok_and(|text| marker_matches(&text, plan));
    let wrote = plan.count
        + if plan.media_fixtures {
            MEDIA_FIXTURE_FOLDERS.len()
        } else {
            0
        };
    marked
        && out.join("Setting").join(SETTINGS_FILE).is_file()
        && library::scan(&out.join("Library"), &app_version())
            .is_ok_and(|scan| scan.themes.len() == wrote && scan.corrupt.len() == CORRUPT)
}

fn generate(plan: &Plan) -> Outcome<()> {
    generate_with(plan, &pool_plan()).map(|_| ())
}

/// Writes the whole synthetic folder, checks it with the library scan and prints the result.
fn generate_with(plan: &Plan, pool: &PoolPlan) -> Outcome<Generated> {
    generate_full(plan, pool, &web_fixtures_dir())
}

/// `generate_with` with the fixtures folder given, so tests can use their own.
fn generate_full(plan: &Plan, pool: &PoolPlan, fixtures_dir: &Path) -> Outcome<Generated> {
    let started = Instant::now();
    let out = std::path::absolute(&plan.out).map_err(io_error("resolve", &plan.out))?;
    for (kind, folder) in guarded_folders() {
        if refuses(&out, &[folder]) {
            return Err(format!(
                "{} is, holds or lies inside {kind}; the generator never writes there and --force does not change that \
                 (run it without SUCROSE_DATA_DIR set)",
                out.display()
            ));
        }
    }
    // Everything that can be refused is refused before the first write: the fixtures are listed and --select is checked
    // against the themes this run will write.
    let fixtures = if plan.web_fixtures {
        fixture_folders(fixtures_dir)?
    } else {
        Vec::new()
    };
    if let Some(select) = &plan.select {
        let synthetic = (0..plan.count).any(|index| theme_name(plan.seed, index) == *select);
        let media = plan.media_fixtures && MEDIA_FIXTURE_FOLDERS.contains(&select.as_str());
        if !synthetic && !media && !fixtures.contains(select) {
            return Err(format!(
                "--select {select}: this run writes no theme with that folder name (fixtures: {}; use --web-fixtures or --media-fixtures)",
                if fixtures.is_empty() {
                    "none".to_owned()
                } else {
                    fixtures.join(", ")
                }
            ));
        }
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
    if is_up_to_date(&out, plan) {
        println!(
            "synth library: up to date ({} themes, seed {}) at {}",
            plan.count,
            plan.seed,
            out.display()
        );
        return Ok(Generated::UpToDate);
    }
    create_dir(&out)?;
    // The marker goes first (unfinished), so a run that dies halfway can be repeated without --force.
    write_file(&out.join(MARKER), b"incomplete\n")?;
    for dir in [&library_dir, &setting_dir, &pool_dir] {
        remove_dir(dir)?;
    }
    create_dir(&library_dir)?;

    let PoolFiles { thumbnails, previews } = write_pool(&pool_dir, plan, pool)?;
    let mut names = BTreeSet::new();
    for index in 0..plan.count + CORRUPT {
        if !names.insert(theme_name(plan.seed, index)) {
            return Err("two themes got the same folder name; use another --seed".to_owned());
        }
    }
    for fixture in &fixtures {
        if names.contains(fixture) {
            return Err(format!(
                "the fixture folder '{fixture}' has the name of a synthetic theme; use another --seed"
            ));
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

    for fixture in &fixtures {
        let dir = library_dir.join(fixture);
        copy_tree(&fixtures_dir.join(fixture), &dir)?;
        fill_missing_images(&dir, &thumbnails[0], &previews[0])?;
    }
    if plan.media_fixtures {
        if let Some(clash) = fixtures
            .iter()
            .find(|fixture| MEDIA_FIXTURE_FOLDERS.contains(&fixture.as_str()))
        {
            return Err(format!(
                "the Web fixture folder '{clash}' has the name of an animation fixture"
            ));
        }
        let counter_size = (480, 270);
        let counter = counter_parts();
        let themes: [(&str, WallpaperType, &str, &str, Vec<u8>); 4] = [
            (
                GIF_FIXTURE_FOLDER,
                WallpaperType::Gif,
                GIF_FIXTURE_SOURCE,
                "A 24 frame GIF that draws its frame number, made by cargo xtask synth-library --media-fixtures.",
                encode_counter_gif()?,
            ),
            (
                APNG_FIXTURE_FOLDER,
                WallpaperType::Gif,
                APNG_FIXTURE_SOURCE,
                "A 24 frame animated PNG that draws its frame number, made by cargo xtask synth-library --media-fixtures.",
                media::apng(counter_size, &counter)?,
            ),
            (
                WEBP_FIXTURE_FOLDER,
                WallpaperType::Gif,
                WEBP_FIXTURE_SOURCE,
                "A 24 frame animated WebP that draws its frame number, made by cargo xtask synth-library --media-fixtures.",
                media::animated_webp(counter_size, &counter)?,
            ),
            (
                VIDEO_FIXTURE_FOLDER,
                WallpaperType::Video,
                VIDEO_FIXTURE_SOURCE,
                "A video theme whose clip is placeholder bytes, made by cargo xtask synth-library --media-fixtures; copy a real clip over it to play it.",
                FAKE_VIDEO.to_vec(),
            ),
        ];
        for (folder, kind, source, description, bytes) in themes {
            let dir = library_dir.join(folder);
            create_dir(&dir)?;
            let mut info = theme_info(0, kind, source, APP_VERSION)?;
            info.title = if kind == WallpaperType::Gif {
                format!("Animation fixture ({folder})")
            } else {
                format!("Media fixture ({folder})")
            };
            info.description = description.to_owned();
            write_file(&dir.join("SucroseInfo.json"), &info_bytes(&info)?)?;
            write_file(&dir.join("Thumbnail.jpg"), &thumbnails[0])?;
            write_file(&dir.join("Preview.gif"), &previews[0])?;
            write_file(&dir.join(source), &bytes)?;
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
    settings.library.selected = plan.select.clone().unwrap_or_default();
    for (kind, engine) in &plan.engines {
        set_engine(&mut settings, *kind, *engine);
    }
    settings.library.delete_corrupt = false;
    settings.general.app_visible = false;
    settings.warehouse.hint_tray_icon = false;
    let bytes = settings
        .to_bytes()
        .map_err(|error| format!("cannot write the settings: {error}"))?;
    write_atomic(&setting_dir.join(SETTINGS_FILE), &bytes).map_err(io_error("write", &setting_dir))?;

    let scan = library::scan(&library_dir, &app_version()).map_err(io_error("scan", &library_dir))?;
    let expected = plan.count
        + fixtures.len()
        + if plan.media_fixtures {
            MEDIA_FIXTURE_FOLDERS.len()
        } else {
            0
        };
    if scan.themes.len() != expected || scan.corrupt.len() != CORRUPT {
        return Err(format!(
            "the library scan found {} themes and {} corrupt folders, expected {expected} and {CORRUPT}",
            scan.themes.len(),
            scan.corrupt.len(),
        ));
    }
    write_file(&out.join(MARKER), marker_for(plan).as_bytes())?;
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
    Ok(Generated::Written)
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn median(mut sizes: Vec<usize>) -> usize {
        sizes.sort_unstable();
        let middle = sizes.len() / 2;
        if sizes.len() % 2 == 1 {
            sizes[middle]
        } else {
            usize::midpoint(sizes[middle - 1], sizes[middle])
        }
    }

    #[test]
    fn synth_theme_name_is_25_alphanumerics_and_stable() {
        let name = theme_name(1, 0);
        assert_eq!(name.len(), NAME_LENGTH);
        assert!(name.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(name, theme_name(1, 0));
        assert_ne!(name, theme_name(2, 0));
        let names: BTreeSet<String> = (0..1000).map(|index| theme_name(1, index)).collect();
        assert_eq!(names.len(), 1000);
    }

    #[test]
    fn synth_pool_plan_targets_follow_the_real_library() {
        let pool = pool_plan();
        assert_eq!((pool.thumbnails.len(), pool.previews.len()), (12, 8));
        let thumbnails: Vec<usize> = pool.thumbnails.iter().map(|spec| spec.target_bytes).collect();
        let previews: Vec<usize> = pool.previews.iter().map(|spec| spec.target_bytes).collect();
        assert!(thumbnails.iter().min().is_some_and(|size| *size <= 120_000));
        assert!(
            thumbnails
                .iter()
                .max()
                .is_some_and(|size| (600_000..=660_000).contains(size))
        );
        assert!((150_000..=250_000).contains(&median(thumbnails)));
        assert!(previews.iter().min().is_some_and(|size| *size <= 250_000));
        assert!(
            previews
                .iter()
                .max()
                .is_some_and(|size| (4_500_000..=5_000_000).contains(size))
        );
        assert!((1_000_000..=2_000_000).contains(&median(previews)));
    }

    #[test]
    fn synth_pool_files_span_the_real_sizes() {
        let pool = pool_plan();
        let thumbnails: Vec<usize> = pool
            .thumbnails
            .iter()
            .enumerate()
            .map(|(index, spec)| make_thumbnail(spec, 1, index).unwrap().len())
            .collect();
        let previews: Vec<usize> = pool
            .previews
            .iter()
            .enumerate()
            .map(|(index, spec)| make_preview(spec, 1, index).unwrap().len())
            .collect();
        // About 100 KB to about 640 KB, median near 190 KB (JPEG thumbnails of the real library).
        assert!(thumbnails.iter().min().is_some_and(|size| *size < 150_000));
        assert!(
            thumbnails
                .iter()
                .max()
                .is_some_and(|size| (500_000..=700_000).contains(size))
        );
        assert!((140_000..=280_000).contains(&median(thumbnails)));
        // About 0.2 MB to about 5 MB, median between 1 and 2 MB (GIF previews); the find checks use MiB and KiB.
        assert!(previews.iter().min().is_some_and(|size| *size < 300 * 1024));
        assert!(
            previews
                .iter()
                .max()
                .is_some_and(|size| (4 * 1024 * 1024..=5_200_000).contains(size))
        );
        assert!((1_000_000..=2_000_000).contains(&median(previews)));
    }

    #[test]
    fn synth_frames_for_reaches_the_target_with_at_least_one_frame() {
        assert_eq!(frames_for(1_000_000, 100_000), 10);
        assert_eq!(frames_for(10, 100), 1);
        assert_eq!(frames_for(250, 100), 3);
    }

    #[test]
    fn synth_marker_matches_only_a_finished_run_with_the_same_count_and_seed() {
        let plan = |count, seed, select: Option<&str>| Plan {
            out: PathBuf::from("x"),
            count,
            seed,
            force: false,
            web_fixtures: false,
            media_fixtures: false,
            select: select.map(str::to_owned),
            engines: Vec::new(),
        };
        let text = marker_text(40, 7);
        assert!(marker_matches(&text, &plan(40, 7, None)));
        assert!(!marker_matches(&text, &plan(41, 7, None)));
        assert!(!marker_matches(&text, &plan(40, 8, None)));
        assert!(!marker_matches("incomplete\n", &plan(40, 7, None)));
        assert!(
            !marker_matches(&text, &plan(40, 7, Some("hello"))),
            "a selection the marker does not record"
        );
        assert!(marker_matches(
            &marker_for(&plan(40, 7, Some("hello"))),
            &plan(40, 7, Some("hello"))
        ));
    }

    #[test]
    fn synth_real_data_folder_is_recognised_even_through_dots_and_case() {
        let real = Path::new("C:/Users/Someone/AppData/Roaming/Sucrose");
        assert!(is_inside(
            Path::new("c:/users/someone/appdata/roaming/sucrose/Library"),
            real
        ));
        assert!(is_inside(
            Path::new("C:/Users/Someone/AppData/Roaming/x/../Sucrose"),
            real
        ));
        assert!(!is_inside(Path::new("C:/Users/Someone/AppData/Roaming/Sucrose2"), real));
    }

    #[test]
    fn synth_second_run_is_up_to_date_and_a_new_count_rewrites() {
        let out = std::env::temp_dir().join(format!("sucrose-synth-test-{}", std::process::id()));
        let other = out.with_extension("other");
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&other);
        let pool = PoolPlan {
            thumbnails: vec![ThumbSpec {
                width: 64,
                height: 36,
                target_bytes: 3_000,
            }],
            previews: vec![PreviewSpec {
                width: 32,
                height: 18,
                target_bytes: 5_000,
            }],
        };
        let plan = Plan {
            out: out.clone(),
            count: 3,
            seed: 7,
            force: false,
            web_fixtures: false,
            media_fixtures: false,
            select: None,
            engines: Vec::new(),
        };
        assert_eq!(generate_with(&plan, &pool).unwrap(), Generated::Written);
        let probe = out.join("Library").join(theme_name(7, 0)).join("SucroseInfo.json");
        let before = fs::metadata(&probe).unwrap().modified().unwrap();
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(generate_with(&plan, &pool).unwrap(), Generated::UpToDate);
        assert_eq!(fs::metadata(&probe).unwrap().modified().unwrap(), before);

        let more = Plan {
            count: 4,
            ..plan.clone()
        };
        assert_eq!(generate_with(&more, &pool).unwrap(), Generated::Written);
        // Four themes, two corrupt folders and the loose file.
        assert_eq!(fs::read_dir(out.join("Library")).unwrap().count(), 7);

        // A folder that is not synthetic output is refused without --force and left alone.
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join("mine.txt"), b"mine").unwrap();
        let refused = Plan {
            out: other.clone(),
            ..plan.clone()
        };
        assert!(generate_with(&refused, &pool).unwrap_err().contains("--force"));
        assert!(other.join("mine.txt").is_file());
        assert!(!other.join("Library").exists());

        fs::remove_dir_all(&out).unwrap();
        fs::remove_dir_all(&other).unwrap();
    }

    /// A fresh folder under the temp folder; the caller removes it.
    fn temp_folder(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sucrose-synth-guard-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn synth_a_verbatim_spelling_of_a_guarded_folder_is_refused() {
        let guarded = temp_folder("verbatim");
        let child = guarded.join("Library");
        assert!(refuses(&child, std::slice::from_ref(&guarded)));
        assert!(refuses(&guarded, std::slice::from_ref(&guarded)));
        if cfg!(windows) {
            // `canonicalize` gives the verbatim form on Windows; a path that does not exist yet is spelled with it too.
            let verbatim = fs::canonicalize(&guarded).unwrap();
            assert!(
                verbatim.to_string_lossy().starts_with(r"\\?\"),
                "{}",
                verbatim.display()
            );
            assert!(refuses(&verbatim.join("Library"), std::slice::from_ref(&guarded)));
            assert!(refuses(
                &verbatim.join("never").join("made"),
                std::slice::from_ref(&guarded)
            ));
            // The guarded side may be the verbatim one as well, and the case does not matter.
            let upper = PathBuf::from(guarded.to_string_lossy().to_uppercase()).join("x");
            assert!(refuses(&upper, &[verbatim]));
            assert_eq!(
                strip_verbatim(Path::new(r"\\?\UNC\srv\share\x")),
                PathBuf::from(r"\\srv\share\x")
            );
            assert_eq!(strip_verbatim(Path::new(r"\\?\C:\x")), PathBuf::from(r"C:\x"));
        } else {
            eprintln!("synth: the verbatim-prefix half of this test only applies on Windows");
        }
        // A folder next to the guarded one is not refused.
        let other = temp_folder("verbatim-other");
        assert!(!refuses(&other.join("Library"), std::slice::from_ref(&guarded)));
        fs::remove_dir_all(&guarded).unwrap();
        fs::remove_dir_all(&other).unwrap();
    }

    #[test]
    fn synth_a_folder_that_holds_a_guarded_folder_is_refused() {
        let outer = temp_folder("holds");
        let guarded = outer.join("AppData").join("Sucrose");
        fs::create_dir_all(&guarded).unwrap();
        assert!(refuses(&outer, std::slice::from_ref(&guarded)));
        assert!(refuses(&outer.join("AppData"), std::slice::from_ref(&guarded)));
        assert!(refuses(
            &outer.join("AppData").join("x").join("..").join("Sucrose"),
            std::slice::from_ref(&guarded)
        ));
        assert!(!refuses(
            &outer.join("AppData").join("Sucrose2"),
            std::slice::from_ref(&guarded)
        ));
        fs::remove_dir_all(&outer).unwrap();
    }

    #[cfg(windows)]
    fn make_dir_link(target: &Path, link: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_dir(target, link)
    }

    #[cfg(unix)]
    fn make_dir_link(target: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[test]
    fn synth_a_link_into_a_guarded_folder_is_refused() {
        let guarded = temp_folder("link-guarded");
        let elsewhere = temp_folder("link-elsewhere");
        let link = elsewhere.join("looks-harmless");
        if let Err(error) = make_dir_link(&guarded, &link) {
            eprintln!("synth: skipped, cannot create a directory symlink here: {error}");
            fs::remove_dir_all(&guarded).unwrap();
            fs::remove_dir_all(&elsewhere).unwrap();
            return;
        }
        assert!(refuses(&link, std::slice::from_ref(&guarded)));
        assert!(refuses(&link.join("Library"), std::slice::from_ref(&guarded)));
        // On Unix a ".." after the link goes to the guarded folder's parent, which holds the guarded folder; Win32 removes
        // the ".." before it looks at the link, so there the OS and the guard both end up beside the link.
        if cfg!(unix) {
            assert!(refuses(&link.join(".."), std::slice::from_ref(&guarded)));
        }
        assert!(!refuses(&elsewhere.join("plain"), std::slice::from_ref(&guarded)));
        // Remove the link itself, never what it points to.
        let _ = fs::remove_dir(&link).or_else(|_| fs::remove_file(&link));
        fs::remove_dir_all(&guarded).unwrap();
        fs::remove_dir_all(&elsewhere).unwrap();
    }

    #[test]
    fn synth_generate_refuses_the_folders_it_guards_even_with_force() {
        let guarded = temp_folder("generate");
        let pool = PoolPlan {
            thumbnails: Vec::new(),
            previews: Vec::new(),
        };
        let plan = Plan {
            out: guarded.join("Library"),
            count: 1,
            seed: 1,
            force: true,
            web_fixtures: false,
            media_fixtures: false,
            select: None,
            engines: Vec::new(),
        };
        // The real data folder of this machine is guarded by name, so a plan inside it is refused whatever else is true.
        if let Some(real) = paths::legacy_data_dir() {
            let inside = Plan {
                out: real.join("synth-guard-test"),
                ..plan.clone()
            };
            let error = generate_with(&inside, &pool).unwrap_err();
            assert!(error.contains("real Sucrose data folder"), "{error}");
            assert!(!inside.out.exists());
            // The new flags do not open the guard either.
            let with_flags = Plan {
                out: real.join("synth-guard-test"),
                web_fixtures: true,
                media_fixtures: false,
                select: Some("hello".to_owned()),
                ..plan.clone()
            };
            let error = generate_with(&with_flags, &pool).unwrap_err();
            assert!(error.contains("real Sucrose data folder"), "{error}");
            assert!(!with_flags.out.exists());
        }
        fs::remove_dir_all(&guarded).unwrap();
    }

    fn tiny_pool() -> PoolPlan {
        PoolPlan {
            thumbnails: vec![ThumbSpec {
                width: 64,
                height: 36,
                target_bytes: 3_000,
            }],
            previews: vec![PreviewSpec {
                width: 32,
                height: 18,
                target_bytes: 5_000,
            }],
        }
    }

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|text| (*text).to_owned()).collect()
    }

    /// A scratch fixtures folder with one Web theme that carries no Thumbnail or Preview, like the committed one.
    fn scratch_fixtures(tag: &str) -> PathBuf {
        let dir = temp_folder(tag);
        let hello = dir.join("hello");
        fs::create_dir_all(&hello).unwrap();
        fs::write(
            hello.join("SucroseInfo.json"),
            br#"{"AppVersion":"26.6.19.0","Version":"1.0.0.0","Title":"hello","Thumbnail":"thumbnail.jpg","Preview":"preview.gif","Description":"d","Author":"a","Contact":"c","Type":2,"Source":"index.html"}"#,
        )
        .unwrap();
        fs::write(hello.join("index.html"), b"<!doctype html><title>hello</title>").unwrap();
        dir
    }

    fn plan_for(out: &Path, web_fixtures: bool, select: Option<&str>) -> Plan {
        Plan {
            out: out.to_path_buf(),
            count: 2,
            seed: 5,
            force: false,
            web_fixtures,
            media_fixtures: false,
            select: select.map(str::to_owned),
            engines: Vec::new(),
        }
    }

    #[test]
    fn synth_parse_reads_web_fixtures_and_select() {
        let plan = Plan::parse(&strings(&[
            "--out",
            "x",
            "--count",
            "2",
            "--web-fixtures",
            "--select",
            "hello",
        ]))
        .unwrap();
        assert!(plan.web_fixtures);
        assert_eq!(plan.select.as_deref(), Some("hello"));
        let plain = Plan::parse(&strings(&["--out", "x", "--count", "2"])).unwrap();
        assert!(!plain.web_fixtures);
        assert_eq!(plain.select, None);
        assert!(Plan::parse(&strings(&["--out", "x", "--count", "2", "--select"])).is_err());
    }

    #[test]
    fn synth_web_fixtures_are_copied_next_to_the_synthetic_themes_and_selected() {
        let fixtures = scratch_fixtures("fixtures-copy");
        let out = temp_folder("fixtures-copy-out");
        let plan = plan_for(&out, true, Some("hello"));
        assert_eq!(
            generate_full(&plan, &tiny_pool(), &fixtures).unwrap(),
            Generated::Written
        );

        let hello = out.join("Library").join("hello");
        assert!(hello.join("index.html").is_file());
        assert!(
            hello.join("thumbnail.jpg").is_file(),
            "the missing Thumbnail is written from the pool"
        );
        assert!(
            hello.join("preview.gif").is_file(),
            "the missing Preview is written from the pool"
        );
        let scan = library::scan(&out.join("Library"), &app_version()).unwrap();
        assert_eq!(scan.themes.len(), 3, "two synthetic themes and the fixture");
        assert_eq!(scan.corrupt.len(), CORRUPT);
        let settings = fs::read_to_string(out.join("Setting").join(SETTINGS_FILE)).unwrap();
        assert!(settings.contains("\"Selected\": \"hello\""), "{settings}");
        // A run with --web-fixtures is never "up to date", so a changed fixture is copied again.
        assert_eq!(
            generate_full(&plan, &tiny_pool(), &fixtures).unwrap(),
            Generated::Written
        );
        fs::remove_dir_all(&fixtures).unwrap();
        fs::remove_dir_all(&out).unwrap();
    }

    #[test]
    fn synth_parse_reads_media_fixtures() {
        let plan = Plan::parse(&strings(&[
            "--out",
            "x",
            "--count",
            "2",
            "--media-fixtures",
            "--select",
            "gif-anim",
        ]))
        .unwrap();
        assert!(plan.media_fixtures);
        assert_eq!(plan.select.as_deref(), Some("gif-anim"));
        assert!(
            !Plan::parse(&strings(&["--out", "x", "--count", "2"]))
                .unwrap()
                .media_fixtures
        );
    }

    #[test]
    fn synth_the_animation_fixture_is_24_frames_of_50_ms_and_the_same_bytes_every_time() {
        use image::{AnimationDecoder, codecs::gif::GifDecoder};

        let bytes = encode_counter_gif().unwrap();
        assert_eq!(bytes, encode_counter_gif().unwrap(), "no randomness");
        let frames = GifDecoder::new(io::Cursor::new(bytes))
            .unwrap()
            .into_frames()
            .collect_frames()
            .unwrap();
        assert_eq!(frames.len(), 24);
        for frame in &frames {
            assert_eq!((frame.buffer().width(), frame.buffer().height()), (480, 270));
            assert_eq!(frame.delay().numer_denom_ms(), (50, 1));
        }
        assert_ne!(frames[0].buffer(), frames[1].buffer(), "every frame differs");
    }

    #[test]
    fn synth_media_fixtures_write_the_gif_theme_and_select_it() {
        let fixtures = scratch_fixtures("media-fixtures");
        let out = temp_folder("media-fixtures-out");
        let mut plan = plan_for(&out, false, Some("gif-anim"));
        plan.media_fixtures = true;
        plan.engines = vec![(WallpaperType::Gif, EngineType::XavierLive)];
        assert_eq!(
            generate_full(&plan, &tiny_pool(), &fixtures).unwrap(),
            Generated::Written
        );

        let theme = out.join("Library").join("gif-anim");
        assert!(theme.join("animation.gif").is_file());
        assert!(theme.join("Thumbnail.jpg").is_file());
        assert!(theme.join("Preview.gif").is_file());
        for (folder, source) in [
            ("apng-anim", "animation.png"),
            ("webp-anim", "animation.webp"),
            ("video-clip", "clip.mp4"),
        ] {
            assert!(out.join("Library").join(folder).join(source).is_file(), "{folder}");
        }
        let scan = library::scan(&out.join("Library"), &app_version()).unwrap();
        assert_eq!(
            scan.themes.len(),
            6,
            "two synthetic themes, three animations and the clip"
        );
        let kinds: Vec<(String, WallpaperType)> = scan
            .themes
            .iter()
            .filter(|theme| MEDIA_FIXTURE_FOLDERS.contains(&theme.folder.as_str()))
            .map(|theme| (theme.folder.clone(), theme.info.kind))
            .collect();
        assert!(
            kinds.contains(&("apng-anim".to_owned(), WallpaperType::Gif)),
            "{kinds:?}"
        );
        assert!(
            kinds.contains(&("webp-anim".to_owned(), WallpaperType::Gif)),
            "{kinds:?}"
        );
        assert!(
            kinds.contains(&("video-clip".to_owned(), WallpaperType::Video)),
            "{kinds:?}"
        );
        assert_eq!(scan.corrupt.len(), CORRUPT);
        let settings = fs::read_to_string(out.join("Setting").join(SETTINGS_FILE)).unwrap();
        assert!(settings.contains("\"Selected\": \"gif-anim\""), "{settings}");
        assert!(settings.contains("XavierLive"), "{settings}");
        // The marker records the fixture, so a second identical run is up to date and a run without it rewrites.
        assert_eq!(
            generate_full(&plan, &tiny_pool(), &fixtures).unwrap(),
            Generated::UpToDate
        );
        let mut without = plan_for(&out, false, None);
        without.engines = plan.engines.clone();
        assert_eq!(
            generate_full(&without, &tiny_pool(), &fixtures).unwrap(),
            Generated::Written
        );
        assert!(!out.join("Library").join("gif-anim").exists());
        // Without the flag the animation cannot be selected.
        let refused = generate_full(&plan_for(&out, false, Some("gif-anim")), &tiny_pool(), &fixtures).unwrap_err();
        assert!(refused.contains("--select"), "{refused}");
        fs::remove_dir_all(&fixtures).unwrap();
        fs::remove_dir_all(&out).unwrap();
    }

    #[test]
    fn synth_select_must_name_a_theme_the_run_wrote_and_writes_nothing_otherwise() {
        let fixtures = scratch_fixtures("fixtures-select");
        let base = temp_folder("fixtures-select-out");
        for (label, plan) in [
            ("unknown folder", plan_for(&base.join("a"), true, Some("nope"))),
            (
                "fixture without --web-fixtures",
                plan_for(&base.join("b"), false, Some("hello")),
            ),
            (
                "a corrupt folder",
                plan_for(&base.join("c"), true, Some(&theme_name(5, 2))),
            ),
        ] {
            let error = generate_full(&plan, &tiny_pool(), &fixtures).unwrap_err();
            assert!(error.contains("--select"), "{label}: {error}");
            assert!(!plan.out.exists(), "{label}: nothing is written");
        }
        // A synthetic theme can be selected without fixtures.
        let name = theme_name(5, 1);
        let ok = plan_for(&base.join("d"), false, Some(&name));
        generate_full(&ok, &tiny_pool(), &fixtures).unwrap();
        let settings = fs::read_to_string(base.join("d").join("Setting").join(SETTINGS_FILE)).unwrap();
        assert!(settings.contains(&format!("\"Selected\": \"{name}\"")), "{settings}");
        // The same run again is up to date (the marker records the selection); another selection rewrites.
        assert_eq!(
            generate_full(&ok, &tiny_pool(), &fixtures).unwrap(),
            Generated::UpToDate
        );
        let other = plan_for(&base.join("d"), false, Some(&theme_name(5, 0)));
        assert_eq!(
            generate_full(&other, &tiny_pool(), &fixtures).unwrap(),
            Generated::Written
        );
        fs::remove_dir_all(&fixtures).unwrap();
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn synth_parse_reads_repeatable_engine_options() {
        let plan = Plan::parse(&strings(&[
            "--out",
            "x",
            "--count",
            "2",
            "--engine",
            "Web=WebViewLive",
            "--engine",
            "Video=MpvPlayerLive",
        ]))
        .unwrap();
        assert_eq!(
            plan.engines,
            vec![
                (WallpaperType::Web, EngineType::WebViewLive),
                (WallpaperType::Video, EngineType::MpvPlayerLive)
            ]
        );
        let plain = Plan::parse(&strings(&["--out", "x", "--count", "2"])).unwrap();
        assert!(plain.engines.is_empty());
    }

    #[test]
    fn synth_engine_outside_the_legacy_list_or_an_unknown_name_is_an_error() {
        for value in [
            "Web=MpvPlayerLive",
            "Foo=CefSharpLive",
            "Web=Nope",
            "Web",
            "Application=WebViewLive",
            "web=WebViewLive",
        ] {
            let error = Plan::parse(&strings(&["--out", "x", "--count", "2", "--engine", value])).unwrap_err();
            assert!(error.contains("--engine"), "{value}: {error}");
        }
        assert!(Plan::parse(&strings(&["--out", "x", "--count", "2", "--engine"])).is_err());
    }

    #[test]
    fn synth_engine_is_written_to_the_settings_and_a_bad_one_writes_nothing() {
        let fixtures = scratch_fixtures("fixtures-engine");
        let base = temp_folder("fixtures-engine-out");
        let args = |out: &Path, engine: &str| {
            strings(&[
                "--out",
                &out.to_string_lossy(),
                "--count",
                "2",
                "--seed",
                "5",
                "--engine",
                engine,
            ])
        };
        // A refused option stops at the parse: nothing is created.
        let refused = base.join("refused");
        assert!(Plan::parse(&args(&refused, "Web=MpvPlayerLive")).is_err());
        assert!(!refused.exists());

        let out = base.join("ok");
        let plan = Plan::parse(&args(&out, "Web=WebViewLive")).unwrap();
        generate_full(&plan, &tiny_pool(), &fixtures).unwrap();
        let settings = fs::read_to_string(out.join("Setting").join(SETTINGS_FILE)).unwrap();
        assert!(settings.contains("\"Web\": \"WebViewLive\""), "{settings}");
        // The marker records the engine: the same run is up to date, another engine rewrites.
        assert_eq!(
            generate_full(&plan, &tiny_pool(), &fixtures).unwrap(),
            Generated::UpToDate
        );
        let other = Plan::parse(&args(&out, "Web=CefSharpLive")).unwrap();
        assert_eq!(
            generate_full(&other, &tiny_pool(), &fixtures).unwrap(),
            Generated::Written
        );
        let settings = fs::read_to_string(out.join("Setting").join(SETTINGS_FILE)).unwrap();
        assert!(settings.contains("\"Web\": \"CefSharpLive\""), "{settings}");
        fs::remove_dir_all(&fixtures).unwrap();
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn synth_the_committed_fixture_theme_hello_is_a_valid_web_theme() {
        let out = temp_folder("fixtures-committed");
        let plan = plan_for(&out, true, Some("hello"));
        generate_with(&plan, &tiny_pool()).unwrap();
        let scan = library::scan(&out.join("Library"), &app_version()).unwrap();
        let hello = scan
            .themes
            .iter()
            .find(|theme| theme.folder == "hello")
            .expect("hello is a valid theme");
        assert_eq!(hello.info.kind, WallpaperType::Web);
        assert_eq!(hello.info.source, "index.html");
        fs::remove_dir_all(&out).unwrap();
    }
}
