//! `cargo xtask measure`: sample the running wallpaper processes and report RAM, CPU and GPU (D-26, D-27, MEAS-01).
//!
//! The task only reads process statistics. It never starts, stops or signals a process; the maintainer starts the
//! C# app for its run. Windows GPU counters are read by running the OS tool `typeperf` (no FFI); on other
//! systems the GPU columns are n/a.
//!
//! Numbers: working set is the resident set, "private (commit)" is the process's private bytes (sysinfo reports
//! `PROCESS_MEMORY_COUNTERS_EX.PrivateUsage` on Windows), CPU is a percentage of the whole machine (sysinfo's
//! percent of one core divided by the logical CPU count) and GPU is, per process and sample, the maximum over engine
//! types like Task Manager shows it. Totals are computed per sample over the processes present, then averaged.
//!
//! Adapter: per measured process the GPU adapter it used is the LUID with the most engine utilization over the run
//! (the evidence is the OS counter, never the adapter enumeration order), named through the platform adapter list.
//! `--row` refuses a pair of results that did not run on one adapter (G-02-6). `dwm.exe` is sampled for information
//! and never added to a total.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sucrose_platform::GpuAdapter;
use sysinfo::{CpuRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

const USAGE: &str = "usage:
  cargo xtask measure --side rust|csharp [--warmup <s>] [--duration <s>] [--out <json>]
  cargo xtask measure --row <csharp.json> <rust.json> --scenario <text> --notes <text>
  cargo xtask measure --side portal [--seconds <n>] [--interval-ms <ms>] [--out <json>]
  cargo xtask measure --portal-summary <json> [--warmup <s>] [--rounds <n>] [--idle <s>]
  cargo xtask measure --portal-summary <json> --portal-log <Portal.log> [--rounds <n>]
  cargo xtask measure --portal-row <real.json> <synth.json> [--warmup <s>] [--rounds <n>] [--idle <s>]
  cargo xtask measure --portal-row <real.json> <synth.json> --portal-logs <real.log> <synth.log> [--rounds <n>]

  --side      rust: every sucrose.exe (service and --render renderers); csharp: Sucrose.Launcher,
              Sucrose.Live.MpvPlayer and Sucrose.Backgroundog; portal: the sucrose-portal.exe process
  --warmup    seconds to wait before sampling (default 60); with --portal-summary/--portal-row the
              warm-up window of the run (default 20)
  --duration  seconds to sample, once per second (default 300)
  --out       result file (default target/measure/<side>.json)
  --row       print the docs/MEASUREMENTS.md table row for two result files
  --seconds   portal: seconds to sample (default 240); --interval-ms: sample spacing (default 500)
  --portal-summary  first-page value, end and peak of each round, peak and final value of a portal series
  --rounds    rounds between the warm-up and the idle window (default 3); --idle: final idle window (default 20)
  --portal-row      print the docs/MEASUREMENTS.md row for the real-library and synthetic-library series
  --portal-log      split the rounds by the Portal's own 'page shown' debug lines (start the Portal with
                    SUCROSE_LOG=info,sucrose_portal::library=debug) instead of fixed windows: the first-page value
                    is the median of the 10 s before the first page change, a round ends at the first page-1 event
                    after the last page, its end value is the median of the next 5 s, the final value runs from 10 s
                    after the last round to 1 s before the series ends (at least 20 s of idle needed); --portal-logs
                    takes the real-library log first and the synthetic-library log second
";
const DEFAULT_PORTAL_SECONDS: u64 = 240;
const DEFAULT_PORTAL_INTERVAL_MS: u64 = 500;
const DEFAULT_PORTAL_WARMUP: u64 = 20;
const DEFAULT_PORTAL_ROUNDS: usize = 3;
const DEFAULT_PORTAL_IDLE: u64 = 20;
/// The first-page value is the median of this many seconds at the end of the warm-up.
const FIRST_PAGE_WINDOW_MS: u64 = 10_000;
/// With the page log, a round's end value is the median of this many ms after its closing page-1 event.
const ROUND_END_WINDOW_MS: u64 = 5_000;
/// With the page log, the final value starts this many ms after the last round's end ...
const FINAL_SKIP_MS: i64 = 10_000;
/// ... and stops this many ms before the series ends.
const FINAL_TAIL_MS: i64 = 1_000;
/// With the page log, the series must run at least this long after the last round's end (the maintainer idles on page 1).
const MIN_LOG_IDLE_MS: i64 = 20_000;
/// The message of the Portal's page event and the target that logs it (apps/sucrose-portal/src/library/mod.rs).
const PAGE_SHOWN: &str = "page shown";
const PAGE_LOG_TARGET: &str = "sucrose_portal::library";
/// How a portal series ended: the planned time ran out, or the Portal process went away.
const ENDED_BY_TIME: &str = "time";
const ENDED_BY_EXIT: &str = "portal-exit";

const MB: f64 = 1024.0 * 1024.0;
const DEFAULT_WARMUP: u64 = 60;
const DEFAULT_DURATION: u64 = 300;
const GPU_ENGINE_COUNTER: &str = r"\GPU Engine(*)\Utilization Percentage";
const GPU_MEMORY_COUNTER: &str = r"\GPU Process Memory(*)\Dedicated Usage";
/// Integrated GPUs report their memory here (Dedicated Usage stays 0), so it is measured too.
const GPU_SHARED_COUNTER: &str = r"\GPU Process Memory(*)\Shared Usage";
/// Image name of the desktop compositor, sampled for information only.
const DWM_IMAGE: &str = "dwm.exe";
/// Name used when a LUID is not in the platform adapter list.
const UNKNOWN_ADAPTER: &str = "unknown adapter";
/// C# roles that make up the "MpvPlayer + Backgroundog" subtotal (D-27).
const CSHARP_SUBTOTAL_ROLES: [&str; 2] = ["mpv-player", "backgroundog"];

/// Which application's processes are measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Rust,
    Csharp,
    /// The Portal (`sucrose-portal.exe`) on its own: sampled by `--side portal` as a time series.
    Portal,
}

impl Side {
    fn parse(text: &str) -> Result<Self, String> {
        match text {
            "rust" => Ok(Self::Rust),
            "csharp" => Ok(Self::Csharp),
            "portal" => Ok(Self::Portal),
            other => Err(format!("--side must be rust, csharp or portal, not '{other}'")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Csharp => "csharp",
            Self::Portal => "portal",
        }
    }
}

#[derive(Debug)]
struct SampleArgs {
    side: Side,
    warmup: u64,
    duration: u64,
    out: Option<String>,
}

#[derive(Debug)]
struct RowArgs {
    csharp: String,
    rust: String,
    scenario: String,
    notes: String,
}

/// The D-25 scenario shape: a warm-up on page 1, `rounds` rounds of paging, a final idle window.
#[derive(Debug)]
pub struct PortalShape {
    warmup: u64,
    rounds: usize,
    idle: u64,
}

#[derive(Debug)]
struct PortalArgs {
    seconds: u64,
    interval_ms: u64,
    out: Option<String>,
}

#[derive(Debug)]
struct PortalSummaryArgs {
    path: String,
    /// The Portal log that splits the rounds; None keeps the fixed windows of `shape`.
    log: Option<String>,
    shape: PortalShape,
}

#[derive(Debug)]
struct PortalRowArgs {
    real: String,
    synth: String,
    /// The real-library and synthetic-library Portal logs; None keeps the fixed windows of `shape`.
    logs: Option<(String, String)>,
    shape: PortalShape,
}

#[derive(Debug)]
enum Task {
    Sample(SampleArgs),
    Row(RowArgs),
    Portal(PortalArgs),
    PortalSummary(PortalSummaryArgs),
    PortalRow(PortalRowArgs),
}

fn value<'a>(it: &mut std::slice::Iter<'a, String>, flag: &str) -> Result<&'a str, String> {
    it.next()
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} needs a value"))
}

fn seconds(text: &str, flag: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .map_err(|_| format!("{flag} must be a whole number of seconds, not '{text}'"))
}

fn count(text: &str, flag: &str) -> Result<usize, String> {
    text.parse::<usize>()
        .map_err(|_| format!("{flag} must be a whole number, not '{text}'"))
}

fn parse_args(args: &[String]) -> Result<Task, String> {
    let mut side = None;
    let mut warmup = None;
    let mut duration = None;
    let mut out = None;
    let mut row: Option<(String, String)> = None;
    let mut scenario = None;
    let mut notes = None;
    let mut portal_seconds = None;
    let mut interval_ms = None;
    let mut rounds = None;
    let mut idle = None;
    let mut summary = None;
    let mut portal_row: Option<(String, String)> = None;
    let mut portal_log: Option<String> = None;
    let mut portal_logs: Option<(String, String)> = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--side" => side = Some(Side::parse(value(&mut it, "--side")?)?),
            "--warmup" => warmup = Some(seconds(value(&mut it, "--warmup")?, "--warmup")?),
            "--duration" => duration = Some(seconds(value(&mut it, "--duration")?, "--duration")?),
            "--out" => out = Some(value(&mut it, "--out")?.to_owned()),
            "--row" => {
                let csharp = value(&mut it, "--row")?.to_owned();
                let rust = value(&mut it, "--row")?.to_owned();
                row = Some((csharp, rust));
            }
            "--scenario" => scenario = Some(value(&mut it, "--scenario")?.to_owned()),
            "--notes" => notes = Some(value(&mut it, "--notes")?.to_owned()),
            "--seconds" => portal_seconds = Some(seconds(value(&mut it, "--seconds")?, "--seconds")?),
            "--interval-ms" => interval_ms = Some(seconds(value(&mut it, "--interval-ms")?, "--interval-ms")?),
            "--rounds" => rounds = Some(count(value(&mut it, "--rounds")?, "--rounds")?),
            "--idle" => idle = Some(seconds(value(&mut it, "--idle")?, "--idle")?),
            "--portal-summary" => summary = Some(value(&mut it, "--portal-summary")?.to_owned()),
            "--portal-row" => {
                let real = value(&mut it, "--portal-row")?.to_owned();
                let synth = value(&mut it, "--portal-row")?.to_owned();
                portal_row = Some((real, synth));
            }
            "--portal-log" => portal_log = Some(value(&mut it, "--portal-log")?.to_owned()),
            "--portal-logs" => {
                let real = value(&mut it, "--portal-logs")?.to_owned();
                let synth = value(&mut it, "--portal-logs")?.to_owned();
                portal_logs = Some((real, synth));
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    let modes = [row.is_some(), side.is_some(), summary.is_some(), portal_row.is_some()];
    if modes.iter().filter(|m| **m).count() > 1 {
        return Err("--side, --row, --portal-summary and --portal-row cannot be combined".to_owned());
    }
    if portal_log.is_some() && summary.is_none() {
        return Err("--portal-log belongs to --portal-summary".to_owned());
    }
    if portal_logs.is_some() && portal_row.is_none() {
        return Err("--portal-logs belongs to --portal-row".to_owned());
    }
    if (portal_log.is_some() || portal_logs.is_some()) && (warmup.is_some() || idle.is_some()) {
        return Err(
            "with a Portal log the rounds, the first page and the final value come from the log; --warmup and --idle do not apply"
                .to_owned(),
        );
    }
    let portal_flags = portal_seconds.is_some() || interval_ms.is_some();
    let shape_flags = rounds.is_some() || idle.is_some();
    let shape = || -> Result<PortalShape, String> {
        let rounds = rounds.unwrap_or(DEFAULT_PORTAL_ROUNDS);
        if rounds == 0 {
            return Err("--rounds must be at least 1".to_owned());
        }
        Ok(PortalShape {
            warmup: warmup.unwrap_or(DEFAULT_PORTAL_WARMUP),
            rounds,
            idle: idle.unwrap_or(DEFAULT_PORTAL_IDLE),
        })
    };
    if let Some(path) = summary {
        if portal_flags || duration.is_some() {
            return Err("--portal-summary takes --portal-log or --warmup/--idle, and --rounds only".to_owned());
        }
        return Ok(Task::PortalSummary(PortalSummaryArgs {
            path,
            log: portal_log,
            shape: shape()?,
        }));
    }
    if let Some((real, synth)) = portal_row {
        if portal_flags || duration.is_some() {
            return Err("--portal-row takes --portal-logs or --warmup/--idle, and --rounds only".to_owned());
        }
        return Ok(Task::PortalRow(PortalRowArgs {
            real,
            synth,
            logs: portal_logs,
            shape: shape()?,
        }));
    }
    if portal_flags && side != Some(Side::Portal) {
        return Err("--seconds and --interval-ms belong to --side portal".to_owned());
    }
    if shape_flags {
        return Err("--rounds and --idle belong to --portal-summary and --portal-row".to_owned());
    }
    match (row, side) {
        (Some((csharp, rust)), None) => Ok(Task::Row(RowArgs {
            csharp,
            rust,
            scenario: scenario.ok_or("--row needs --scenario")?,
            notes: notes.ok_or("--row needs --notes")?,
        })),
        (None, Some(Side::Portal)) => {
            if warmup.is_some() || duration.is_some() {
                return Err("--warmup and --duration do not apply to --side portal; use --seconds".to_owned());
            }
            let seconds = portal_seconds.unwrap_or(DEFAULT_PORTAL_SECONDS);
            let interval_ms = interval_ms.unwrap_or(DEFAULT_PORTAL_INTERVAL_MS);
            if seconds == 0 {
                return Err("--seconds must be at least 1".to_owned());
            }
            if interval_ms == 0 {
                return Err("--interval-ms must be at least 1".to_owned());
            }
            if seconds * 1000 < interval_ms {
                return Err("--seconds must cover at least one --interval-ms".to_owned());
            }
            Ok(Task::Portal(PortalArgs {
                seconds,
                interval_ms,
                out,
            }))
        }
        (None, Some(side)) => {
            let duration = duration.unwrap_or(DEFAULT_DURATION);
            if duration == 0 {
                return Err("--duration must be at least 1".to_owned());
            }
            Ok(Task::Sample(SampleArgs {
                side,
                warmup: warmup.unwrap_or(DEFAULT_WARMUP),
                duration,
                out,
            }))
        }
        (None, None) => Err("give --side, --row, --portal-summary or --portal-row".to_owned()),
        (Some(_), Some(_)) => Err("--row and --side cannot be combined".to_owned()),
    }
}

/// `cargo xtask measure ...`
pub fn run(args: &[String]) -> ExitCode {
    let result = parse_args(args).and_then(|task| match task {
        Task::Sample(sample_args) => sample(&sample_args),
        Task::Row(row_args) => print_row(&row_args),
        Task::Portal(portal_args) => sample_portal(&portal_args),
        Task::PortalSummary(summary_args) => print_portal_summary(&summary_args),
        Task::PortalRow(row_args) => print_portal_row(&row_args),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("failed: {message}");
            eprint!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// The role of a process for a side, from its image name and command line; None when it is not measured.
/// Rust renderers share the service's image name and are told apart by `--render`.
pub fn classify(side: Side, name: &str, cmd: &[String]) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".exe").unwrap_or(&lower);
    match side {
        Side::Rust if stem == "sucrose" => {
            let renderer = cmd.iter().any(|a| a == "--render" || a.starts_with("--render="));
            Some(if renderer { "renderer" } else { "service" })
        }
        Side::Csharp => match stem {
            "sucrose.launcher" => Some("launcher"),
            "sucrose.live.mpvplayer" => Some("mpv-player"),
            "sucrose.backgroundog" => Some("backgroundog"),
            _ => None,
        },
        Side::Portal if stem == "sucrose-portal" => Some("portal"),
        Side::Rust | Side::Portal => None,
    }
}

/// One matching process.
#[derive(Debug, Clone)]
struct Found {
    pid: u32,
    name: String,
    role: &'static str,
    exe_profile: Option<String>,
}

/// Whether an executable path sits in a cargo `release` or `debug` folder.
pub fn exe_profile(path: &str) -> Option<String> {
    let parts: Vec<String> = path.split(['\\', '/']).map(str::to_ascii_lowercase).collect();
    if parts.iter().any(|p| p == "release") {
        Some("release".to_owned())
    } else if parts.iter().any(|p| p == "debug") {
        Some("debug".to_owned())
    } else {
        None
    }
}

fn refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_memory()
        .with_cpu()
        .with_cmd(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet)
}

fn scan(system: &mut System, side: Side) -> Vec<Found> {
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());
    let mut found: Vec<Found> = system
        .processes()
        .values()
        .filter_map(|process| {
            let name = process.name().to_string_lossy().into_owned();
            let cmd: Vec<String> = process.cmd().iter().map(|a| a.to_string_lossy().into_owned()).collect();
            let role = classify(side, &name, &cmd)?;
            let exe_profile = process.exe().and_then(|p| exe_profile(&p.to_string_lossy()));
            Some(Found {
                pid: process.pid().as_u32(),
                name,
                role,
                exe_profile,
            })
        })
        .collect();
    found.sort_by_key(|f| f.pid);
    found
}

/// One per-second reading of one process: MB, percent of the machine and the GPU numbers once attached.
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    ws_mb: f64,
    commit_mb: f64,
    cpu_pct: f64,
    gpu_pct: Option<f64>,
    gpu_mem_mb: Option<f64>,
    gpu_shared_mb: Option<f64>,
}

/// Averages and peaks of a process or a group of processes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    pub ws_avg_mb: f64,
    pub ws_peak_mb: f64,
    pub commit_avg_mb: f64,
    pub commit_peak_mb: f64,
    pub cpu_avg_pct: f64,
    pub cpu_peak_pct: f64,
    pub gpu_avg_pct: Option<f64>,
    pub gpu_mem_avg_mb: Option<f64>,
    /// Shared GPU memory (what integrated GPUs use), average in MB.
    #[serde(default)]
    pub gpu_shared_avg_mb: Option<f64>,
    pub samples: usize,
}

#[derive(Default)]
struct Acc {
    sum: f64,
    peak: f64,
    n: usize,
}

impl Acc {
    fn add(&mut self, v: f64) {
        self.sum += v;
        self.peak = self.peak.max(v);
        self.n += 1;
    }

    fn avg(&self) -> f64 {
        if self.n == 0 { 0.0 } else { self.sum / self.n as f64 }
    }
}

/// Metrics of a group: at every tick the processes present are summed, then the ticks are averaged.
/// A single process is a group of one.
pub fn metrics(series: &[&[Option<Point>]]) -> Metrics {
    let ticks = series.iter().map(|s| s.len()).max().unwrap_or(0);
    let (mut ws, mut commit, mut cpu, mut gpu, mut gpu_mem, mut gpu_shared) = (
        Acc::default(),
        Acc::default(),
        Acc::default(),
        Acc::default(),
        Acc::default(),
        Acc::default(),
    );
    for tick in 0..ticks {
        let present: Vec<&Point> = series.iter().filter_map(|s| s.get(tick)?.as_ref()).collect();
        if present.is_empty() {
            continue;
        }
        ws.add(present.iter().map(|p| p.ws_mb).sum());
        commit.add(present.iter().map(|p| p.commit_mb).sum());
        cpu.add(present.iter().map(|p| p.cpu_pct).sum());
        let gpus: Vec<f64> = present.iter().filter_map(|p| p.gpu_pct).collect();
        if !gpus.is_empty() {
            gpu.add(gpus.iter().sum());
        }
        let mems: Vec<f64> = present.iter().filter_map(|p| p.gpu_mem_mb).collect();
        if !mems.is_empty() {
            gpu_mem.add(mems.iter().sum());
        }
        let shared: Vec<f64> = present.iter().filter_map(|p| p.gpu_shared_mb).collect();
        if !shared.is_empty() {
            gpu_shared.add(shared.iter().sum());
        }
    }
    Metrics {
        ws_avg_mb: ws.avg(),
        ws_peak_mb: ws.peak,
        commit_avg_mb: commit.avg(),
        commit_peak_mb: commit.peak,
        cpu_avg_pct: cpu.avg(),
        cpu_peak_pct: cpu.peak,
        gpu_avg_pct: (gpu.n > 0).then(|| gpu.avg()),
        gpu_mem_avg_mb: (gpu_mem.n > 0).then(|| gpu_mem.avg()),
        gpu_shared_avg_mb: (gpu_shared.n > 0).then(|| gpu_shared.avg()),
        samples: ws.n,
    }
}

// ---- typeperf CSV ----

/// Splits one CSV line with double-quoted fields (`""` is a quote inside a field).
pub fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = line.trim_end_matches(['\r', '\n']).chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut current)),
            other => current.push(other),
        }
    }
    fields.push(current);
    fields
}

/// A parsed typeperf run: one instance name per counter column and one row of values per sample.
#[derive(Debug, Default)]
pub struct CounterTable {
    instances: Vec<String>,
    rows: Vec<Vec<f64>>,
}

/// The instance of a counter path such as `\\HOST\GPU Engine(pid_4_luid_..._engtype_3D)\Utilization Percentage`.
fn instance_of(path: &str) -> Option<&str> {
    let open = path.find('(')?;
    let close = path.rfind(")\\")?;
    path.get(open + 1..close)
}

/// A number as typeperf prints it; a decimal comma (some locales) is accepted. Blank or unparseable is 0.
fn number(text: &str) -> f64 {
    let t = text.trim();
    t.parse::<f64>()
        .or_else(|_| t.replace(',', ".").parse::<f64>())
        .unwrap_or(0.0)
}

/// Parses typeperf's CSV output: a `(PDH-CSV` header, data rows, then trailer lines that are ignored.
pub fn parse_typeperf(text: &str) -> Result<CounterTable, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let header = lines
        .by_ref()
        .find(|l| l.starts_with("\"(PDH-CSV"))
        .ok_or("typeperf printed no CSV header (the counter may not exist on this machine)")?;
    let columns = parse_csv_line(header);
    let instances: Vec<String> = columns
        .iter()
        .skip(1)
        .map(|path| instance_of(path).unwrap_or_default().to_owned())
        .collect();
    let mut rows = Vec::new();
    for line in lines {
        if !line.starts_with('"') {
            break;
        }
        let fields = parse_csv_line(line);
        if fields.len() != columns.len() {
            continue;
        }
        rows.push(fields.iter().skip(1).map(|f| number(f)).collect());
    }
    Ok(CounterTable { instances, rows })
}

/// A GPU counter instance name split into its parts.
#[derive(Debug, PartialEq, Eq)]
pub struct GpuInstance {
    pub pid: u32,
    pub luid: String,
    pub engine_type: Option<String>,
}

/// Parses `pid_<pid>_luid_<hi>_<lo>_phys_<n>[_eng_<n>_engtype_<type>]`.
pub fn parse_gpu_instance(name: &str) -> Option<GpuInstance> {
    let rest = name.strip_prefix("pid_")?;
    let (pid_text, rest) = rest.split_once('_')?;
    let pid = pid_text.parse().ok()?;
    let rest = rest.strip_prefix("luid_")?;
    let luid = rest.split("_phys_").next()?.to_owned();
    let engine_type = name.split_once("_engtype_").map(|(_, t)| t.to_owned());
    Some(GpuInstance { pid, luid, engine_type })
}

/// Per pid and sample, the maximum over engine types (summed within a type of one adapter) of the utilization.
pub fn gpu_utilization(table: &CounterTable, pids: &BTreeSet<u32>) -> BTreeMap<u32, Vec<f64>> {
    let parsed: Vec<Option<GpuInstance>> = table.instances.iter().map(|i| parse_gpu_instance(i)).collect();
    let mut out: BTreeMap<u32, Vec<f64>> = pids.iter().map(|p| (*p, Vec::new())).collect();
    for row in &table.rows {
        let mut per_pid: BTreeMap<u32, BTreeMap<(String, String), f64>> = BTreeMap::new();
        for (instance, value) in parsed.iter().zip(row) {
            let Some(instance) = instance else { continue };
            if !pids.contains(&instance.pid) {
                continue;
            }
            let key = (instance.luid.clone(), instance.engine_type.clone().unwrap_or_default());
            *per_pid.entry(instance.pid).or_default().entry(key).or_insert(0.0) += value;
        }
        for (pid, series) in &mut out {
            let best = per_pid
                .get(pid)
                .map_or(0.0, |types| types.values().fold(0.0_f64, |a, b| a.max(*b)));
            series.push(best.min(100.0));
        }
    }
    out
}

/// Per pid and sample, the dedicated GPU memory in MB summed over the process's adapters.
pub fn gpu_memory(table: &CounterTable, pids: &BTreeSet<u32>) -> BTreeMap<u32, Vec<f64>> {
    let parsed: Vec<Option<GpuInstance>> = table.instances.iter().map(|i| parse_gpu_instance(i)).collect();
    let mut out: BTreeMap<u32, Vec<f64>> = pids.iter().map(|p| (*p, Vec::new())).collect();
    for row in &table.rows {
        for (pid, series) in &mut out {
            let bytes: f64 = parsed
                .iter()
                .zip(row)
                .filter(|(instance, _)| instance.as_ref().is_some_and(|i| i.pid == *pid))
                .map(|(_, v)| *v)
                .sum();
            series.push(bytes / MB);
        }
    }
    out
}

/// The LUID of the adapter the given processes used most over the run: per LUID the utilization summed over every
/// sample, engine type and pid. `None` when the processes show no utilization at all.
pub fn adapter_of_pids(table: &CounterTable, pids: &BTreeSet<u32>) -> Option<String> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for row in &table.rows {
        for (instance, value) in table.instances.iter().zip(row) {
            let Some(instance) = parse_gpu_instance(instance) else {
                continue;
            };
            if pids.contains(&instance.pid) {
                *totals.entry(instance.luid).or_insert(0.0) += value;
            }
        }
    }
    totals
        .into_iter()
        .filter(|(_, total)| *total > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(luid, _)| luid)
}

/// The adapter named `luid` from the platform's adapter list, or "unknown adapter" with the LUID kept.
pub fn adapter_ref(luid: &str, adapters: &[GpuAdapter]) -> AdapterRef {
    AdapterRef {
        luid: luid.to_owned(),
        name: adapters
            .iter()
            .find(|a| a.luid.eq_ignore_ascii_case(luid))
            .map_or_else(|| UNKNOWN_ADAPTER.to_owned(), |a| a.name.clone()),
    }
}

fn run_typeperf(counter: &str, seconds: u64) -> Result<String, String> {
    let output = Command::new("typeperf")
        .arg(counter)
        .args(["-si", "1", "-sc", &seconds.to_string()])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run typeperf: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!("typeperf exited with {}", output.status))
    }
}

// ---- report ----

/// A GPU adapter as recorded in a result file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterRef {
    /// The LUID in typeperf's text form (`0x00000000_0x0000A1B2`); it changes at every boot.
    pub luid: String,
    pub name: String,
}

/// One measured process in the result file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessReport {
    pub pid: u32,
    pub name: String,
    pub role: String,
    pub exe_profile: Option<String>,
    pub metrics: Metrics,
    /// The adapter with the most engine utilization over the run; none for a process that used no GPU or a result
    /// written before adapters were recorded.
    #[serde(default)]
    pub gpu_adapter: Option<AdapterRef>,
}

/// `dwm.exe` GPU use over the run, for information: never part of a total or subtotal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DwmReport {
    pub gpu_avg_pct: f64,
    pub gpu_adapter: Option<AdapterRef>,
}

/// A named group of the measured processes, summed per sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtotal {
    pub label: String,
    pub roles: Vec<String>,
    pub metrics: Metrics,
}

/// The result file of one run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub side: String,
    /// UTC time the sampling started, ISO 8601.
    pub started: String,
    pub warmup_secs: u64,
    pub duration_secs: u64,
    pub logical_cpus: usize,
    /// `release`, `debug` or `mixed` when the executables' folders are known.
    pub build: Option<String>,
    pub gpu_available: bool,
    pub processes: Vec<ProcessReport>,
    pub total: Metrics,
    pub subtotal: Option<Subtotal>,
    /// The compositor's GPU use next to the measured processes (information only).
    #[serde(default)]
    pub dwm: Option<DwmReport>,
    pub warnings: Vec<String>,
}

/// UTC civil date and time of a Unix timestamp as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn iso_utc(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64;
    let rem = unix_secs % 86_400;
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

/// UTC time of a Unix timestamp in milliseconds as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
pub fn iso_utc_ms(unix_ms: u64) -> String {
    let secs = iso_utc(unix_ms / 1000);
    format!("{}.{:03}Z", secs.trim_end_matches('Z'), unix_ms % 1000)
}

/// Days from 1970-01-01 to a civil date (Howard Hinnant's algorithm, the inverse of `iso_utc`).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let doy = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Milliseconds since the Unix epoch of an RFC 3339 UTC time (`...Z`) with or without a fraction of a second (the
/// Portal log writes microseconds, the series writes milliseconds, older series whole seconds); None for any other shape.
pub fn parse_utc_ms(text: &str) -> Option<i64> {
    fn number(text: &str) -> Option<i64> {
        if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        text.parse().ok()
    }
    let (date, time) = text.strip_suffix('Z')?.split_once('T')?;
    let (clock, fraction) = time.split_once('.').unwrap_or((time, ""));
    let mut date = date.splitn(3, '-');
    let (year, month, day) = (number(date.next()?)?, number(date.next()?)?, number(date.next()?)?);
    let mut clock = clock.splitn(3, ':');
    let (hour, minute, second) = (number(clock.next()?)?, number(clock.next()?)?, number(clock.next()?)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let millis = if fraction.is_empty() {
        0
    } else {
        let digits: String = fraction.chars().chain("00".chars()).take(3).collect();
        if !fraction.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        digits.parse::<i64>().ok()?
    };
    let secs = days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second;
    Some(secs * 1000 + millis)
}

fn now_iso_ms() -> String {
    iso_utc_ms(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(0)),
    )
}

fn now_iso() -> String {
    iso_utc(SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()))
}

fn build_of(found: &[Found]) -> Option<String> {
    let profiles: BTreeSet<&str> = found.iter().filter_map(|f| f.exe_profile.as_deref()).collect();
    match profiles.len() {
        0 => None,
        1 => profiles.into_iter().next().map(str::to_owned),
        _ => Some("mixed".to_owned()),
    }
}

fn sample(args: &SampleArgs) -> Result<(), String> {
    let mut system = System::new().map_err(|e| format!("cannot read system information: {e}"))?;
    system.refresh_cpu_list(CpuRefreshKind::nothing());
    let cpus = system.cpus().len().max(1);

    let first = scan(&mut system, args.side);
    if first.is_empty() {
        return Err(format!(
            "no matching processes: no {} processes are running; start the app first",
            args.side.as_str()
        ));
    }
    println!(
        "{} processes found ({}); warming up for {} s",
        first.len(),
        describe(&first),
        args.warmup
    );
    thread::sleep(Duration::from_secs(args.warmup));

    let found = scan(&mut system, args.side);
    if found.is_empty() {
        return Err("no matching processes: the app exited during the warm-up".to_owned());
    }
    let pids: BTreeSet<u32> = found.iter().map(|f| f.pid).collect();
    let dwm_pids = dwm_pids(&system);
    let pid_list: Vec<Pid> = pids.iter().map(|p| Pid::from_u32(*p)).collect();
    println!("sampling {} s: {}", args.duration, describe(&found));

    let gpu_threads = if cfg!(windows) {
        let secs = args.duration;
        Some((
            thread::spawn(move || run_typeperf(GPU_ENGINE_COUNTER, secs)),
            thread::spawn(move || run_typeperf(GPU_MEMORY_COUNTER, secs)),
            thread::spawn(move || run_typeperf(GPU_SHARED_COUNTER, secs)),
        ))
    } else {
        None
    };

    let started = now_iso();
    let begin = Instant::now();
    let mut series: BTreeMap<u32, Vec<Option<Point>>> = pids.iter().map(|p| (*p, Vec::new())).collect();
    for tick in 1..=args.duration {
        let due = begin + Duration::from_secs(tick);
        thread::sleep(due.saturating_duration_since(Instant::now()));
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&pid_list), true, refresh_kind());
        for (pid, points) in &mut series {
            points.push(system.process(Pid::from_u32(*pid)).map(|p| Point {
                ws_mb: p.memory() as f64 / MB,
                commit_mb: p.virtual_memory() as f64 / MB,
                cpu_pct: f64::from(p.cpu_usage()) / cpus as f64,
                gpu_pct: None,
                gpu_mem_mb: None,
                gpu_shared_mb: None,
            }));
        }
        if tick % 30 == 0 {
            println!("  {tick}/{} s", args.duration);
        }
    }

    let mut warnings = Vec::new();
    let mut gpu_available = false;
    let mut adapter_by_pid: BTreeMap<u32, AdapterRef> = BTreeMap::new();
    let mut dwm = None;
    if let Some((engine, memory, shared)) = gpu_threads {
        let engine = engine.join().map_err(|_| "GPU sampler thread panicked".to_owned())?;
        let memory = memory.join().map_err(|_| "GPU sampler thread panicked".to_owned())?;
        let shared = shared.join().map_err(|_| "GPU sampler thread panicked".to_owned())?;
        match (
            engine.and_then(|t| parse_typeperf(&t)),
            memory.and_then(|t| parse_typeperf(&t)),
            shared.and_then(|t| parse_typeperf(&t)),
        ) {
            (Ok(engine), Ok(memory), Ok(shared)) => {
                gpu_available = true;
                let util = gpu_utilization(&engine, &pids);
                let mem = gpu_memory(&memory, &pids);
                let shared_mem = gpu_memory(&shared, &pids);
                let adapters = sucrose_platform::current().gpu_adapters();
                for pid in &pids {
                    if let Some(luid) = adapter_of_pids(&engine, &BTreeSet::from([*pid])) {
                        adapter_by_pid.insert(*pid, adapter_ref(&luid, &adapters));
                    }
                }
                dwm = dwm_report(&engine, &dwm_pids, &adapters);
                for (pid, points) in &mut series {
                    for (tick, point) in points.iter_mut().enumerate() {
                        if let Some(point) = point {
                            point.gpu_pct = util.get(pid).and_then(|s| s.get(tick)).copied();
                            point.gpu_mem_mb = mem.get(pid).and_then(|s| s.get(tick)).copied();
                            point.gpu_shared_mb = shared_mem.get(pid).and_then(|s| s.get(tick)).copied();
                        }
                    }
                }
            }
            (engine, memory, shared) => {
                for message in [engine.err(), memory.err(), shared.err()].into_iter().flatten() {
                    warnings.push(format!("GPU numbers unavailable: {message}"));
                }
            }
        }
    } else {
        warnings.push("GPU numbers are only measured on Windows (typeperf)".to_owned());
    }

    let after: BTreeSet<u32> = scan(&mut system, args.side).iter().map(|f| f.pid).collect();
    if after != pids {
        warnings.push(format!(
            "the process set changed during the run (start {pids:?}, end {after:?}); numbers cover the start set only"
        ));
    }
    for (pid, points) in &series {
        let missing = points.iter().filter(|p| p.is_none()).count();
        if missing > 0 {
            warnings.push(format!(
                "pid {pid} was missing from {missing} of {} samples",
                points.len()
            ));
        }
    }

    let all: Vec<&[Option<Point>]> = series.values().map(Vec::as_slice).collect();
    let processes = found
        .iter()
        .map(|f| ProcessReport {
            pid: f.pid,
            name: f.name.clone(),
            role: f.role.to_owned(),
            exe_profile: f.exe_profile.clone(),
            metrics: metrics(&[series[&f.pid].as_slice()]),
            gpu_adapter: adapter_by_pid.get(&f.pid).cloned(),
        })
        .collect();
    let subtotal = (args.side == Side::Csharp).then(|| {
        let group: Vec<&[Option<Point>]> = found
            .iter()
            .filter(|f| CSHARP_SUBTOTAL_ROLES.contains(&f.role))
            .map(|f| series[&f.pid].as_slice())
            .collect();
        Subtotal {
            label: "MpvPlayer + Backgroundog".to_owned(),
            roles: CSHARP_SUBTOTAL_ROLES.iter().map(|r| (*r).to_owned()).collect(),
            metrics: metrics(&group),
        }
    });
    let report = Report {
        side: args.side.as_str().to_owned(),
        started,
        warmup_secs: args.warmup,
        duration_secs: args.duration,
        logical_cpus: cpus,
        build: build_of(&found),
        gpu_available,
        processes,
        total: metrics(&all),
        subtotal,
        dwm,
        warnings,
    };
    print_report(&report);

    let out = args
        .out
        .clone()
        .unwrap_or_else(|| format!("target/measure/{}.json", args.side.as_str()));
    write_report(&report, Path::new(&out))?;
    println!("wrote {out}");
    Ok(())
}

/// The pids of the desktop compositor.
fn dwm_pids(system: &System) -> BTreeSet<u32> {
    system
        .processes()
        .values()
        .filter(|p| p.name().to_string_lossy().eq_ignore_ascii_case(DWM_IMAGE))
        .map(|p| p.pid().as_u32())
        .collect()
}

/// `dwm.exe` GPU average (summed over its processes per sample, like a group) and adapter; none without a dwm process.
fn dwm_report(engine: &CounterTable, dwm_pids: &BTreeSet<u32>, adapters: &[GpuAdapter]) -> Option<DwmReport> {
    if dwm_pids.is_empty() {
        return None;
    }
    let util = gpu_utilization(engine, dwm_pids);
    let ticks = util.values().map(Vec::len).max().unwrap_or(0);
    let mut acc = Acc::default();
    for tick in 0..ticks {
        acc.add(util.values().filter_map(|s| s.get(tick)).sum());
    }
    Some(DwmReport {
        gpu_avg_pct: acc.avg(),
        gpu_adapter: adapter_of_pids(engine, dwm_pids).map(|luid| adapter_ref(&luid, adapters)),
    })
}

fn describe(found: &[Found]) -> String {
    found
        .iter()
        .map(|f| format!("{} {} pid {}", f.role, f.name, f.pid))
        .collect::<Vec<_>>()
        .join(", ")
}

fn write_report(report: &Report, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(report).map_err(|e| format!("cannot encode the result: {e}"))?;
    fs::write(path, json).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

fn read_report(path: &str) -> Result<Report, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path} is not a measure result: {e}"))
}

fn gpu_text(value: Option<f64>, unit: &str) -> String {
    value.map_or_else(|| "n/a".to_owned(), |v| format!("{v:.1}{unit}"))
}

fn adapter_text(adapter: Option<&AdapterRef>) -> String {
    adapter.map_or_else(|| "no adapter".to_owned(), |a| a.name.clone())
}

fn print_report(report: &Report) {
    println!(
        "\n{} side, {} s after a {} s warm-up, {} logical CPUs, build {}",
        report.side,
        report.duration_secs,
        report.warmup_secs,
        report.logical_cpus,
        report.build.as_deref().unwrap_or("unknown")
    );
    println!(
        "{:<13} {:>7} {:>9} {:>9} {:>11} {:>8} {:>8} {:>8} {:>9} {:>10}  GPU adapter",
        "role", "pid", "WS avg", "WS peak", "private avg", "CPU avg", "CPU peak", "GPU avg", "GPU mem", "GPU shared"
    );
    let line = |label: &str, pid: &str, m: &Metrics, adapter: &str| {
        println!(
            "{label:<13} {pid:>7} {:>8.1}M {:>8.1}M {:>10.1}M {:>7.2}% {:>7.2}% {:>8} {:>9} {:>10}  {adapter}",
            m.ws_avg_mb,
            m.ws_peak_mb,
            m.commit_avg_mb,
            m.cpu_avg_pct,
            m.cpu_peak_pct,
            gpu_text(m.gpu_avg_pct, "%"),
            gpu_text(m.gpu_mem_avg_mb, "M"),
            gpu_text(m.gpu_shared_avg_mb, "M"),
        );
    };
    for p in &report.processes {
        line(
            &p.role,
            &p.pid.to_string(),
            &p.metrics,
            &adapter_text(p.gpu_adapter.as_ref()),
        );
    }
    line("total", "", &report.total, "");
    if let Some(sub) = &report.subtotal {
        line("subtotal", "", &sub.metrics, "");
        println!("(subtotal = {})", sub.label);
    }
    match &report.dwm {
        Some(dwm) => println!(
            "dwm.exe GPU avg {:.1}% on {} (information only, not in the totals)",
            dwm.gpu_avg_pct,
            adapter_text(dwm.gpu_adapter.as_ref())
        ),
        None => println!("dwm.exe: not sampled"),
    }
    for warning in &report.warnings {
        println!("warning: {warning}");
    }
}

// ---- MEASUREMENTS row ----

fn cell_metrics(m: &Metrics) -> String {
    let mut text = format!(
        "RAM {:.0} MB (peak {:.0}), private {:.0} MB, CPU {:.2}% (peak {:.1}%)",
        m.ws_avg_mb, m.ws_peak_mb, m.commit_avg_mb, m.cpu_avg_pct, m.cpu_peak_pct
    );
    if let Some(gpu) = m.gpu_avg_pct {
        let _ = write!(text, ", GPU {gpu:.1}%");
    }
    if let (Some(dedicated), Some(shared)) = (m.gpu_mem_avg_mb, m.gpu_shared_avg_mb) {
        let _ = write!(text, ", GPU memory {dedicated:.0} MB dedicated / {shared:.0} MB shared");
    }
    text
}

fn count_role(report: &Report, role: &str) -> usize {
    report.processes.iter().filter(|p| p.role == role).count()
}

/// The C# cell: the total of the measured processes and the MpvPlayer + Backgroundog subtotal (D-27).
pub fn csharp_cell(report: &Report) -> String {
    let mut cell = format!("{} processes: {}", report.processes.len(), cell_metrics(&report.total));
    if let Some(sub) = &report.subtotal {
        let _ = write!(cell, "; {}: {}", sub.label, cell_metrics(&sub.metrics));
    }
    cell
}

/// The Rust cell: the service plus every renderer.
pub fn rust_cell(report: &Report) -> String {
    let renderers = count_role(report, "renderer");
    format!(
        "service + {renderers} renderer{}: {}",
        if renderers == 1 { "" } else { "s" },
        cell_metrics(&report.total)
    )
}

fn clean_cell(text: &str) -> String {
    text.replace('|', "/").replace(['\r', '\n'], " ")
}

/// One docs/MEASUREMENTS.md table row: `| Date | Scenario | C# app | Rust | Notes |`.
pub fn format_row(csharp: &Report, rust: &Report, scenario: &str, notes: &str) -> String {
    let date = rust.started.get(..10).unwrap_or(&rust.started);
    format!(
        "| {date} | {} | {} | {} | {} |",
        clean_cell(scenario),
        clean_cell(&csharp_cell(csharp)),
        clean_cell(&rust_cell(rust)),
        clean_cell(notes)
    )
}

/// The adapters of every process of `role` in a result; Err when there is none of that role or one has no adapter.
fn role_adapters<'a>(report: &'a Report, role: &str, who: &str) -> Result<Vec<&'a AdapterRef>, String> {
    let of_role: Vec<&ProcessReport> = report.processes.iter().filter(|p| p.role == role).collect();
    if of_role.is_empty() {
        return Err(format!(
            "the {who} result has no {role} process, so its GPU adapter is unknown; the comparison is not valid, measure again"
        ));
    }
    of_role
        .into_iter()
        .map(|p| {
            p.gpu_adapter.as_ref().ok_or_else(|| {
                format!(
                    "the {who} result has no GPU adapter for {role} pid {} (the process used no GPU or the result was written \
                     by an older measure); the comparison is not valid, measure again",
                    p.pid
                )
            })
        })
        .collect()
}

/// The adapter both sides ran on: the C# MpvPlayer and every Rust renderer must share one LUID. Err names both adapters
/// when they differ and when a result has no adapter, so a cross-GPU comparison can never be recorded (G-02-6).
pub fn same_adapter(csharp: &Report, rust: &Report) -> Result<AdapterRef, String> {
    let c = role_adapters(csharp, "mpv-player", "C#")?;
    let r = role_adapters(rust, "renderer", "Rust")?;
    let first = c[0];
    if c.iter().chain(r.iter()).all(|a| a.luid == first.luid) {
        return Ok(first.clone());
    }
    let names = |list: &[&AdapterRef]| -> String {
        let mut names: Vec<&str> = Vec::new();
        for a in list {
            if !names.contains(&a.name.as_str()) {
                names.push(a.name.as_str());
            }
        }
        names.join(" / ")
    };
    let (c_names, r_names) = (names(&c), names(&r));
    let mut message = format!(
        "the two sides ran on different GPUs (C#: {c_names}, Rust: {r_names}); the comparison is not valid, measure again"
    );
    if c_names == r_names {
        message.push_str(
            " (the adapter names are equal but the LUIDs differ; LUIDs change at every boot, so measure both sides in one boot session)",
        );
    }
    Err(message)
}

fn dwm_text(report: &Report) -> String {
    report
        .dwm
        .as_ref()
        .map_or_else(|| "n/a".to_owned(), |d| format!("{:.1} %", d.gpu_avg_pct))
}

/// The table row for a pair of results, only when both ran on the same adapter; the adapter and the dwm.exe GPU of
/// both sides are appended to the notes.
pub fn checked_row(csharp: &Report, rust: &Report, scenario: &str, notes: &str) -> Result<String, String> {
    let adapter = same_adapter(csharp, rust)?;
    let gpu = format!(
        "GPU: {}, both sides. dwm.exe GPU: C# {}, Rust {}.",
        adapter.name,
        dwm_text(csharp),
        dwm_text(rust)
    );
    // A note that ends without a full stop must not run into the GPU sentence.
    let notes = match notes.trim().trim_end_matches(['.', ';', ',']).trim_end() {
        "" => gpu,
        notes => format!("{notes}. {gpu}"),
    };
    Ok(format_row(csharp, rust, scenario, &notes))
}

fn print_row(args: &RowArgs) -> Result<(), String> {
    let csharp = read_report(&args.csharp)?;
    let rust = read_report(&args.rust)?;
    if csharp.side != "csharp" || rust.side != "rust" {
        return Err("--row takes the C# result first and the Rust result second".to_owned());
    }
    println!("{}", checked_row(&csharp, &rust, &args.scenario, &args.notes)?);
    Ok(())
}

// ---- Portal memory over time (D-25) ----

/// One reading of the Portal process.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PortalSample {
    pub elapsed_ms: u64,
    pub working_set_mb: f64,
    pub private_mb: f64,
}

/// The result file of `--side portal`: a time series of one process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalSeries {
    pub side: String,
    pub started: String,
    pub interval_ms: u64,
    pub pid: u32,
    /// `release` or `debug` when the executable's folder is known.
    pub build: Option<String>,
    /// False when the process ended before the planned time.
    pub complete: bool,
    /// `time` when the planned time ran out, `portal-exit` when the Portal went away first (the maintainer closed it
    /// after the idle window); absent in files from before the page-log split.
    #[serde(default)]
    pub ended_by: Option<String>,
    pub samples: Vec<PortalSample>,
}

/// Statistics of one metric over a D-25 run, in MB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortalSummary {
    pub first_page: f64,
    pub round_end: Vec<f64>,
    pub round_peak: Vec<f64>,
    pub peak: f64,
    pub final_value: f64,
}

/// The median of a non-empty list.
fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        values[mid]
    } else {
        f64::midpoint(values[mid - 1], values[mid])
    }
}

fn maximum(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::MIN, f64::max)
}

/// Statistics of `(elapsed ms, value)` points in the D-25 shape. Windows are half-open on time, `(from, to]`, because
/// the sampler stamps a sample with the end of its interval: the warm-up is `(0, warmup]`, the idle window is the last
/// `idle` seconds and the rounds split the time between them evenly. `first_page` is the median of the last 10 s of the
/// warm-up, `round_end` the last sample of each round, `round_peak` its maximum, `peak` the maximum of the whole run
/// and `final_value` the median of the idle window.
pub fn portal_summary(
    points: &[(u64, f64)],
    warmup_s: u64,
    rounds: usize,
    idle_s: u64,
) -> Result<PortalSummary, String> {
    if rounds == 0 {
        return Err("--rounds must be at least 1".to_owned());
    }
    if warmup_s == 0 {
        return Err("--warmup must be at least 1".to_owned());
    }
    if idle_s == 0 {
        return Err("--idle must be at least 1".to_owned());
    }
    let end = points.last().map(|p| p.0).ok_or("the series has no samples")?;
    let warmup_ms = warmup_s * 1000;
    let idle_ms = idle_s * 1000;
    if warmup_ms + idle_ms >= end {
        return Err(format!(
            "the series covers {} s, which leaves no time for the rounds after a {warmup_s} s warm-up and a {idle_s} s idle window",
            end / 1000
        ));
    }
    let window = |from: u64, to: u64| -> Vec<f64> {
        points
            .iter()
            .filter(|(t, _)| *t > from && *t <= to)
            .map(|(_, v)| *v)
            .collect()
    };
    let first = window(warmup_ms.saturating_sub(FIRST_PAGE_WINDOW_MS), warmup_ms);
    if first.is_empty() {
        return Err("the series has no samples in the last 10 s of the warm-up".to_owned());
    }
    let span = end - warmup_ms - idle_ms;
    let mut round_end = Vec::with_capacity(rounds);
    let mut round_peak = Vec::with_capacity(rounds);
    for round in 0..rounds {
        let from = warmup_ms + span * round as u64 / rounds as u64;
        let to = warmup_ms + span * (round as u64 + 1) / rounds as u64;
        let values = window(from, to);
        let last = *values
            .last()
            .ok_or_else(|| format!("round {} has no samples", round + 1))?;
        round_end.push(last);
        round_peak.push(maximum(&values));
    }
    let idle = window(end - idle_ms, end);
    if idle.is_empty() {
        return Err("the idle window has no samples".to_owned());
    }
    let all: Vec<f64> = points.iter().map(|(_, v)| *v).collect();
    Ok(PortalSummary {
        first_page: median(first),
        round_end,
        round_peak,
        peak: maximum(&all),
        final_value: median(idle),
    })
}

/// The working-set and private-bytes summaries of a series.
fn series_summaries(series: &PortalSeries, shape: &PortalShape) -> Result<(PortalSummary, PortalSummary), String> {
    let pick = |f: fn(&PortalSample) -> f64| -> Vec<(u64, f64)> {
        series.samples.iter().map(|s| (s.elapsed_ms, f(s))).collect()
    };
    let working_set = portal_summary(&pick(|s| s.working_set_mb), shape.warmup, shape.rounds, shape.idle)?;
    let private = portal_summary(&pick(|s| s.private_mb), shape.warmup, shape.rounds, shape.idle)?;
    Ok((working_set, private))
}

fn mb_list(values: &[f64]) -> String {
    values.iter().map(|v| format!("{v:.0}")).collect::<Vec<_>>().join(" / ")
}

fn portal_cell(label: &str, series: &PortalSeries, shape: &PortalShape) -> Result<String, String> {
    let (ws, private) = series_summaries(series, shape)?;
    Ok(format_cell(label, &ws, &private))
}

fn format_cell(label: &str, ws: &PortalSummary, private: &PortalSummary) -> String {
    format!(
        "{label}: working set first page {:.0} MB, peak {:.0}, final {:.0} (round ends {}, round peaks {}); \
         private first page {:.0} MB, peak {:.0}, final {:.0}",
        ws.first_page,
        ws.peak,
        ws.final_value,
        mb_list(&ws.round_end),
        mb_list(&ws.round_peak),
        private.first_page,
        private.peak,
        private.final_value
    )
}

fn require_release(series: &PortalSeries, who: &str) -> Result<(), String> {
    if !series.complete {
        return Err(format!(
            "the {who} series ended before its planned time (the Portal exited); measure again"
        ));
    }
    require_build(series, who)
}

fn require_build(series: &PortalSeries, who: &str) -> Result<(), String> {
    match series.build.as_deref() {
        Some("release") => Ok(()),
        other => Err(format!(
            "the {who} series was not measured on a release build ({}); the numbers are not comparable, measure again",
            other.unwrap_or("build unknown")
        )),
    }
}

/// The MEASUREMENTS row of the two scenarios: Rust column only (the C# Portal is not a comparison target here).
pub fn portal_row(real: &PortalSeries, synth: &PortalSeries, shape: &PortalShape) -> Result<String, String> {
    require_release(real, "real-library")?;
    require_release(synth, "synthetic-library")?;
    let date = synth.started.get(..10).unwrap_or(&synth.started);
    let cells = [
        portal_cell("real library", real, shape)?,
        portal_cell("500 synthetic themes", synth, shape)?,
    ];
    let notes = format!(
        "Portal process only, sampled every {} ms and {} ms; {} s warm-up on page 1, {} rounds forward and back through every page \
         with hover previews on (by hand), {} s idle on page 1; release build; scratch data folders",
        real.interval_ms, synth.interval_ms, shape.warmup, shape.rounds, shape.idle
    );
    Ok(format!(
        "| {date} | M2 Portal library paging, memory over time (Rust, release build) | – | {} | {} |",
        clean_cell(&cells.join("; ")),
        clean_cell(&notes)
    ))
}

// ---- Rounds from the Portal's page log ----

/// One "page shown" line of the Portal log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageEvent {
    /// Milliseconds since the Unix epoch from `page_events`; milliseconds since the series start after `rebase_events`
    /// (negative for an event before the sampler started).
    pub at_ms: i64,
    /// The page shown, counted from 1.
    pub page: u32,
    /// How many pages the library has, when the line says so.
    pub pages: Option<u32>,
}

/// The "page shown" events of a Portal log (target `sucrose_portal::library`), in file order. Other lines, a line
/// without a readable `page=` and a last line that was cut off (no newline yet) are ignored.
pub fn page_events(log: &str) -> Vec<PageEvent> {
    let complete = if log.ends_with('\n') {
        log
    } else {
        log.rfind('\n').map_or("", |end| &log[..=end])
    };
    complete.lines().filter_map(page_event).collect()
}

fn page_event(line: &str) -> Option<PageEvent> {
    let (stamp, rest) = line.trim_start().split_once(char::is_whitespace)?;
    let at_ms = parse_utc_ms(stamp)?;
    let (head, fields) = rest.split_once(PAGE_SHOWN)?;
    if !head.contains(PAGE_LOG_TARGET) {
        return None;
    }
    let (mut page, mut pages) = (None, None);
    for field in fields.split_whitespace() {
        match field.split_once('=') {
            Some(("page", value)) => page = value.parse::<u32>().ok().filter(|page| *page >= 1),
            Some(("pages", value)) => pages = value.parse::<u32>().ok(),
            _ => {}
        }
    }
    Some(PageEvent {
        at_ms,
        page: page?,
        pages,
    })
}

/// The events with their time counted from the series start (`start_ms`, Unix milliseconds).
pub fn rebase_events(events: &[PageEvent], start_ms: i64) -> Vec<PageEvent> {
    events
        .iter()
        .map(|event| PageEvent {
            at_ms: event.at_ms - start_ms,
            ..*event
        })
        .collect()
}

/// One round of paging: from the first page change after the previous round to its closing page-1 event, in
/// milliseconds since the series start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundSpan {
    pub start_ms: i64,
    pub end_ms: i64,
}

/// The result of splitting a series by the page log: the usual statistics plus where each round was.
#[derive(Debug, Clone, PartialEq)]
pub struct PortalLogSummary {
    pub values: PortalSummary,
    pub rounds: Vec<RoundSpan>,
}

/// Splits time-ordered events into rounds. A round starts at the first event whose page is not 1 and ends at the first
/// page-1 event after the last page was shown. The second value is the start of a round that never closed.
fn split_rounds(events: &[PageEvent]) -> (Vec<RoundSpan>, Option<i64>) {
    let seen_max = events.iter().map(|event| event.page).max().unwrap_or(1);
    let mut rounds = Vec::new();
    let mut start = None;
    let mut reached_last = false;
    for event in events {
        if event.page != 1 && start.is_none() {
            start = Some(event.at_ms);
        }
        let last = event.pages.unwrap_or(seen_max);
        if last > 1 && event.page >= last {
            reached_last = true;
        }
        if event.page == 1 && reached_last {
            if let Some(start_ms) = start {
                rounds.push(RoundSpan {
                    start_ms,
                    end_ms: event.at_ms,
                });
            }
            start = None;
            reached_last = false;
        }
    }
    (rounds, start)
}

/// Statistics of one metric when the rounds come from the Portal's page log (events counted from the series start):
/// `first_page` is the median of the 10 s before the first page change, a round's end value the median of the 5 s
/// after its closing page-1 event, its peak the maximum between its start and its end, `final_value` the median from
/// 10 s after the last round's end to 1 s before the series ends, `peak` the maximum of the whole run. Windows are
/// half-open on time, `(from, to]`, as in `portal_summary`. Refuses, naming what is missing, a log with fewer than
/// `rounds` complete rounds (or paging beyond them), a warm-up shorter than 10 s and less than 20 s of idle at the end.
pub fn portal_log_summary(
    points: &[(u64, f64)],
    events: &[PageEvent],
    rounds: usize,
) -> Result<PortalLogSummary, String> {
    if rounds == 0 {
        return Err("--rounds must be at least 1".to_owned());
    }
    let points: Vec<(i64, f64)> = points
        .iter()
        .map(|(t, v)| (i64::try_from(*t).unwrap_or(i64::MAX), *v))
        .collect();
    let end = points.last().map(|p| p.0).ok_or("the series has no samples")?;
    let mut events = events.to_vec();
    events.sort_by_key(|event| event.at_ms);
    let (spans, open) = split_rounds(&events);
    let first_change =
        spans.first().map(|span| span.start_ms).or(open).ok_or(
            "the page log shows no page change: the maintainer never left page 1, or the log lines are missing",
        )?;
    let window_ms = i64::try_from(FIRST_PAGE_WINDOW_MS).unwrap_or(i64::MAX);
    if first_change < window_ms {
        return Err(format!(
            "the warm-up before the first page change is {:.1} s, shorter than the {} s the first-page value needs",
            first_change as f64 / 1000.0,
            FIRST_PAGE_WINDOW_MS / 1000
        ));
    }
    if spans.len() < rounds {
        return Err(format!(
            "the page log has {} complete rounds, --rounds asks for {rounds}; a round ends at the first page-1 event after \
             the last page was shown",
            spans.len()
        ));
    }
    if spans.len() > rounds {
        return Err(format!(
            "the page log has {} complete rounds, more than --rounds {rounds}; the final window would not be idle, \
             give --rounds {}",
            spans.len(),
            spans.len()
        ));
    }
    if let Some(started) = open {
        return Err(format!(
            "the page log shows paging again from {:.1} s after the last round's end; the final window would not be \
             idle, measure again",
            (started - spans[rounds - 1].end_ms) as f64 / 1000.0
        ));
    }
    for (index, span) in spans.iter().enumerate() {
        if span.end_ms <= 0 || span.end_ms > end {
            return Err(format!(
                "round {} ends outside the series (at {:.1} s of {:.1} s): the log does not belong to this series",
                index + 1,
                span.end_ms as f64 / 1000.0,
                end as f64 / 1000.0
            ));
        }
    }
    let last_end = spans[rounds - 1].end_ms;
    let idle = end - last_end;
    if idle < MIN_LOG_IDLE_MS {
        return Err(format!(
            "the series ends {:.1} s after the last round's end; the final idle window needs at least {} s (close the \
             Portal only after that)",
            idle as f64 / 1000.0,
            MIN_LOG_IDLE_MS / 1000
        ));
    }
    let window = |from: i64, to: i64| -> Vec<f64> {
        points
            .iter()
            .filter(|(t, _)| *t > from && *t <= to)
            .map(|(_, v)| *v)
            .collect()
    };
    let first = window(first_change - window_ms, first_change);
    if first.is_empty() {
        return Err("the series has no samples in the 10 s before the first page change".to_owned());
    }
    let after = i64::try_from(ROUND_END_WINDOW_MS).unwrap_or(i64::MAX);
    let mut round_end = Vec::with_capacity(rounds);
    let mut round_peak = Vec::with_capacity(rounds);
    for (index, span) in spans.iter().enumerate() {
        let settled = window(span.end_ms, span.end_ms + after);
        let inside = window(span.start_ms, span.end_ms);
        if settled.is_empty() || inside.is_empty() {
            return Err(format!("round {} has no samples", index + 1));
        }
        round_end.push(median(settled));
        round_peak.push(maximum(&inside));
    }
    let idle_values = window(last_end + FINAL_SKIP_MS, end - FINAL_TAIL_MS);
    if idle_values.is_empty() {
        return Err("the final idle window has no samples".to_owned());
    }
    let all: Vec<f64> = points.iter().map(|(_, v)| *v).collect();
    Ok(PortalLogSummary {
        values: PortalSummary {
            first_page: median(first),
            round_end,
            round_peak,
            peak: maximum(&all),
            final_value: median(idle_values),
        },
        rounds: spans,
    })
}

/// A series can be split by its log when it ran its time out or ended because the Portal was closed.
fn require_log_series(series: &PortalSeries, who: &str) -> Result<(), String> {
    if series.complete || series.ended_by.as_deref() == Some(ENDED_BY_EXIT) {
        Ok(())
    } else {
        Err(format!(
            "the {who} series ended before its planned time and does not say why (an older file); measure again"
        ))
    }
}

/// The working-set and private-bytes summaries of a series, split by the page log text.
fn log_summaries(
    series: &PortalSeries,
    log: &str,
    rounds: usize,
) -> Result<(PortalLogSummary, PortalLogSummary), String> {
    let start = parse_utc_ms(&series.started)
        .ok_or_else(|| format!("the series start '{}' is not an RFC 3339 UTC time", series.started))?;
    let events = rebase_events(&page_events(log), start);
    if events.is_empty() {
        return Err(format!(
            "the log has no \"{PAGE_SHOWN}\" lines; start the Portal with SUCROSE_LOG=\"info,{PAGE_LOG_TARGET}=debug\""
        ));
    }
    let pick = |f: fn(&PortalSample) -> f64| -> Vec<(u64, f64)> {
        series.samples.iter().map(|s| (s.elapsed_ms, f(s))).collect()
    };
    let working_set = portal_log_summary(&pick(|s| s.working_set_mb), &events, rounds)?;
    let private = portal_log_summary(&pick(|s| s.private_mb), &events, rounds)?;
    Ok((working_set, private))
}

/// The MEASUREMENTS row of the two scenarios with the rounds taken from each scenario's Portal log text.
pub fn portal_row_from_logs(
    real: (&PortalSeries, &str),
    synth: (&PortalSeries, &str),
    rounds: usize,
) -> Result<String, String> {
    require_log_series(real.0, "real-library")?;
    require_log_series(synth.0, "synthetic-library")?;
    require_build(real.0, "real-library")?;
    require_build(synth.0, "synthetic-library")?;
    let cell = |label: &str, (series, log): (&PortalSeries, &str), who: &str| -> Result<String, String> {
        let (ws, private) = log_summaries(series, log, rounds).map_err(|e| format!("{who}: {e}"))?;
        Ok(format_cell(label, &ws.values, &private.values))
    };
    let date = synth.0.started.get(..10).unwrap_or(&synth.0.started);
    let cells = [
        cell("real library", real, "real-library")?,
        cell("500 synthetic themes", synth, "synthetic-library")?,
    ];
    let notes = format!(
        "Portal process only, sampled every {} ms and {} ms; {rounds} rounds forward and back through every page with hover \
         previews on (by hand); the rounds come from the Portal's page log (first page = median of the 10 s before the first \
         page change, round end = median of the 5 s after the closing page-1 event, final = median from 10 s after the last \
         round to 1 s before the end); release build; scratch data folders",
        real.0.interval_ms, synth.0.interval_ms
    );
    Ok(format!(
        "| {date} | M2 Portal library paging, memory over time (Rust, release build) | – | {} | {} |",
        clean_cell(&cells.join("; ")),
        clean_cell(&notes)
    ))
}

/// The offset of local time from UTC in minutes, from `+HHMM`, `+HH:MM` or a whole number of minutes.
fn parse_utc_offset(text: &str) -> Option<i32> {
    let text = text.trim();
    if let Some(sign) = text.chars().next().filter(|c| matches!(c, '+' | '-')) {
        let digits: String = text[1..].chars().filter(|c| *c != ':').collect();
        if digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit()) {
            let hours: i32 = digits[..2].parse().ok()?;
            let minutes: i32 = digits[2..].parse().ok()?;
            let total = hours * 60 + minutes;
            return Some(if sign == '-' { -total } else { total });
        }
    }
    text.parse().ok()
}

/// Local time's offset from UTC now, asked of the OS (PowerShell on Windows, `date` elsewhere); None when unknown.
fn local_utc_offset() -> Option<i32> {
    let output = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[int][TimeZoneInfo]::Local.GetUtcOffset([DateTime]::UtcNow).TotalMinutes",
            ])
            .stderr(Stdio::null())
            .output()
    } else {
        Command::new("date").arg("+%z").stderr(Stdio::null()).output()
    }
    .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_utc_offset(&String::from_utf8_lossy(&output.stdout))
}

/// `HH:MM:SS.t` of a Unix time in milliseconds at the given offset from UTC.
fn clock_text(unix_ms: i64, offset_minutes: i32) -> String {
    let day_ms = (unix_ms + i64::from(offset_minutes) * 60_000).rem_euclid(86_400_000);
    format!(
        "{:02}:{:02}:{:02}.{}",
        day_ms / 3_600_000,
        day_ms % 3_600_000 / 60_000,
        day_ms % 60_000 / 1000,
        day_ms % 1000 / 100
    )
}

fn offset_label(offset_minutes: Option<i32>) -> String {
    match offset_minutes {
        Some(0) => "UTC".to_owned(),
        Some(minutes) => format!(
            "UTC{}{:02}:{:02}",
            if minutes < 0 { '-' } else { '+' },
            minutes.abs() / 60,
            minutes.abs() % 60
        ),
        None => "UTC, the local offset could not be read".to_owned(),
    }
}

/// One line per round with its start and end in local time (UTC when the offset is unknown) and its length, for
/// cross-checking with what the maintainer remembers.
fn round_times(rounds: &[RoundSpan], series_start_ms: i64, offset_minutes: Option<i32>) -> Vec<String> {
    let offset = offset_minutes.unwrap_or(0);
    rounds
        .iter()
        .enumerate()
        .map(|(index, span)| {
            format!(
                "round {}: {} to {} ({:.1} s)",
                index + 1,
                clock_text(series_start_ms + span.start_ms, offset),
                clock_text(series_start_ms + span.end_ms, offset),
                (span.end_ms - span.start_ms) as f64 / 1000.0
            )
        })
        .collect()
}

fn write_portal_series(series: &PortalSeries, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(series).map_err(|e| format!("cannot encode the result: {e}"))?;
    fs::write(path, json).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

fn read_portal_series(path: &str) -> Result<PortalSeries, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let series: PortalSeries =
        serde_json::from_str(&text).map_err(|e| format!("{path} is not a portal series: {e}"))?;
    if series.side != "portal" {
        return Err(format!("{path} is a {} result, not a portal series", series.side));
    }
    Ok(series)
}

/// Samples the Portal's working set and private bytes every `--interval-ms` for `--seconds` (D-25). Only reads
/// process statistics: the maintainer starts the Portal and pages through it by hand.
fn sample_portal(args: &PortalArgs) -> Result<(), String> {
    let mut system = System::new().map_err(|e| format!("cannot read system information: {e}"))?;
    let found = scan(&mut system, Side::Portal);
    let Some(target) = found.first() else {
        return Err("no matching processes: sucrose-portal.exe is not running; start the Portal first".to_owned());
    };
    if found.len() > 1 {
        println!("{} Portal processes found; sampling pid {}", found.len(), target.pid);
    }
    let pid = Pid::from_u32(target.pid);
    let ticks = args.seconds * 1000 / args.interval_ms;
    println!(
        "sampling {} (pid {}) every {} ms for {} s",
        target.name, target.pid, args.interval_ms, args.seconds
    );
    let kind = ProcessRefreshKind::nothing().with_memory();
    let started = now_iso_ms();
    let begin = Instant::now();
    let mut samples = Vec::new();
    let mut complete = true;
    for tick in 1..=ticks {
        let elapsed_ms = tick * args.interval_ms;
        thread::sleep((begin + Duration::from_millis(elapsed_ms)).saturating_duration_since(Instant::now()));
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, kind);
        let Some(process) = system.process(pid) else {
            complete = false;
            break;
        };
        samples.push(PortalSample {
            elapsed_ms,
            working_set_mb: process.memory() as f64 / MB,
            private_mb: process.virtual_memory() as f64 / MB,
        });
        if elapsed_ms.is_multiple_of(30_000) {
            println!("  {} / {} s", elapsed_ms / 1000, args.seconds);
        }
    }
    let series = PortalSeries {
        side: Side::Portal.as_str().to_owned(),
        started,
        interval_ms: args.interval_ms,
        pid: target.pid,
        build: build_of(&found),
        complete,
        ended_by: Some((if complete { ENDED_BY_TIME } else { ENDED_BY_EXIT }).to_owned()),
        samples,
    };
    let out = args
        .out
        .clone()
        .unwrap_or_else(|| format!("target/measure/{}.json", Side::Portal.as_str()));
    write_portal_series(&series, Path::new(&out))?;
    println!("wrote {out} ({} samples)", series.samples.len());
    if !complete {
        // Closing the Portal after the idle window is how a page-log run ends, so this is not a failure.
        println!(
            "the Portal (pid {}) exited after {} s; the series is saved with ended_by {ENDED_BY_EXIT} (the page-log \
             summary accepts it, the fixed-window summary does not)",
            target.pid,
            series.samples.last().map_or(0, |s| s.elapsed_ms) / 1000
        );
    }
    Ok(())
}

fn print_summary_line(label: &str, summary: &PortalSummary) {
    println!(
        "{label}: first page {:.1}, round end {}, round peak {}, peak {:.1}, final {:.1}",
        summary.first_page,
        mb_list(&summary.round_end),
        mb_list(&summary.round_peak),
        summary.peak,
        summary.final_value
    );
}

fn print_portal_log_summary(args: &PortalSummaryArgs, log_path: &str) -> Result<(), String> {
    let series = read_portal_series(&args.path)?;
    require_log_series(&series, &args.path)?;
    let log = fs::read_to_string(log_path).map_err(|e| format!("cannot read {log_path}: {e}"))?;
    let (working_set, private) = log_summaries(&series, &log, args.shape.rounds)?;
    println!(
        "{}: {} samples every {} ms, build {}, started {}, ended by {}; rounds from the page log {log_path} (MB)",
        args.path,
        series.samples.len(),
        series.interval_ms,
        series.build.as_deref().unwrap_or("unknown"),
        series.started,
        series.ended_by.as_deref().unwrap_or("unknown"),
    );
    let offset = local_utc_offset();
    let start = parse_utc_ms(&series.started).unwrap_or(0);
    println!("round times, local clock ({}):", offset_label(offset));
    for line in round_times(&working_set.rounds, start, offset) {
        println!("  {line}");
    }
    print_summary_line("working set", &working_set.values);
    print_summary_line("private    ", &private.values);
    Ok(())
}

fn print_portal_summary(args: &PortalSummaryArgs) -> Result<(), String> {
    if let Some(log) = &args.log {
        return print_portal_log_summary(args, log);
    }
    let series = read_portal_series(&args.path)?;
    if !series.complete {
        eprintln!(
            "warning: {} ended before its planned time (the Portal exited)",
            args.path
        );
    }
    let (working_set, private) = series_summaries(&series, &args.shape)?;
    println!(
        "{}: {} samples every {} ms, build {}; warm-up {} s, {} rounds, idle {} s (MB)",
        args.path,
        series.samples.len(),
        series.interval_ms,
        series.build.as_deref().unwrap_or("unknown"),
        args.shape.warmup,
        args.shape.rounds,
        args.shape.idle
    );
    print_summary_line("working set", &working_set);
    print_summary_line("private    ", &private);
    Ok(())
}

fn print_portal_row(args: &PortalRowArgs) -> Result<(), String> {
    let real = read_portal_series(&args.real)?;
    let synth = read_portal_series(&args.synth)?;
    if let Some((real_log, synth_log)) = &args.logs {
        let read = |path: &String| fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"));
        let (real_text, synth_text) = (read(real_log)?, read(synth_log)?);
        println!(
            "{}",
            portal_row_from_logs((&real, &real_text), (&synth, &synth_text), args.shape.rounds)?
        );
        return Ok(());
    }
    println!("{}", portal_row(&real, &synth, &args.shape)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(ws: f64, cpu: f64, gpu: Option<f64>) -> Option<Point> {
        Some(Point {
            ws_mb: ws,
            commit_mb: ws * 2.0,
            cpu_pct: cpu,
            gpu_pct: gpu,
            gpu_mem_mb: None,
            gpu_shared_mb: None,
        })
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn csv_line_handles_quotes_and_commas() {
        let fields = parse_csv_line("\"a,b\",\"say \"\"hi\"\"\",\"3\"\r\n");
        assert_eq!(fields, vec!["a,b", "say \"hi\"", "3"]);
    }

    #[test]
    fn classify_separates_service_and_renderers() {
        let render = strings(&["sucrose.exe", "--render", "1"]);
        assert_eq!(classify(Side::Rust, "sucrose.exe", &render), Some("renderer"));
        assert_eq!(
            classify(Side::Rust, "Sucrose.exe", &strings(&["sucrose.exe"])),
            Some("service")
        );
        assert_eq!(classify(Side::Rust, "sucrose-portal.exe", &render), None);
        assert_eq!(classify(Side::Rust, "Sucrose.Launcher.exe", &[]), None);
        assert_eq!(
            classify(Side::Csharp, "Sucrose.Live.MpvPlayer.exe", &[]),
            Some("mpv-player")
        );
        assert_eq!(
            classify(Side::Csharp, "Sucrose.Backgroundog.exe", &[]),
            Some("backgroundog")
        );
        assert_eq!(classify(Side::Csharp, "Sucrose.Launcher.exe", &[]), Some("launcher"));
        assert_eq!(classify(Side::Csharp, "Sucrose.Portal.exe", &[]), None);
    }

    #[test]
    fn metrics_sum_present_processes_per_tick() {
        let a = vec![point(100.0, 1.0, Some(10.0)), point(120.0, 3.0, Some(20.0))];
        let b = vec![point(50.0, 0.5, None), None];
        let m = metrics(&[a.as_slice(), b.as_slice()]);
        assert!((m.ws_avg_mb - (150.0 + 120.0) / 2.0).abs() < 1e-9);
        assert!((m.ws_peak_mb - 150.0).abs() < 1e-9);
        assert!((m.commit_avg_mb - (300.0 + 240.0) / 2.0).abs() < 1e-9);
        assert!((m.cpu_avg_pct - (1.5 + 3.0) / 2.0).abs() < 1e-9);
        assert!((m.cpu_peak_pct - 3.0).abs() < 1e-9);
        assert_eq!(m.gpu_avg_pct, Some(15.0));
        assert_eq!(m.samples, 2);
        assert_eq!(metrics(&[]).gpu_avg_pct, None);
    }

    const GPU_CSV: &str = "\"(PDH-CSV 4.0)\",\"\\\\PC\\GPU Engine(pid_10_luid_0x0_0xA_phys_0_eng_0_engtype_3D)\\Utilization Percentage\",\
\"\\\\PC\\GPU Engine(pid_10_luid_0x0_0xA_phys_0_eng_1_engtype_VideoDecode)\\Utilization Percentage\",\
\"\\\\PC\\GPU Engine(pid_99_luid_0x0_0xA_phys_0_eng_0_engtype_3D)\\Utilization Percentage\"\r\n\
\"11/22/2026 10:00:01.000\",\"2.5\",\"7,5\",\"50\"\r\n\
\"11/22/2026 10:00:02.000\",\"4\",\"1\",\"60\"\r\n\
\r\nExiting, please wait...\r\nThe command completed successfully.\r\n";

    #[test]
    fn typeperf_table_gives_max_engine_type_per_pid() {
        let table = parse_typeperf(GPU_CSV).expect("valid csv");
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.instances.len(), 3);
        let pids: BTreeSet<u32> = [10, 99, 7].into_iter().collect();
        let util = gpu_utilization(&table, &pids);
        assert_eq!(util[&10], vec![7.5, 4.0]);
        assert_eq!(util[&99], vec![50.0, 60.0]);
        assert_eq!(util[&7], vec![0.0, 0.0]);
        assert!(parse_typeperf("Error: no such counter\r\n").is_err());
    }

    #[test]
    fn gpu_instance_names_split_into_parts() {
        let a = parse_gpu_instance("pid_4242_luid_0x00000000_0x0000A1B2_phys_0_eng_3_engtype_VideoDecode");
        assert_eq!(
            a,
            Some(GpuInstance {
                pid: 4242,
                luid: "0x00000000_0x0000A1B2".to_owned(),
                engine_type: Some("VideoDecode".to_owned()),
            })
        );
        let m = parse_gpu_instance("pid_7_luid_0x0_0x1_phys_0");
        assert_eq!(m.map(|i| (i.pid, i.engine_type)), Some((7, None)));
        assert_eq!(parse_gpu_instance("garbage"), None);
    }

    #[test]
    fn gpu_memory_sums_adapters_in_mb() {
        let text = "\"(PDH-CSV 4.0)\",\"\\\\PC\\GPU Process Memory(pid_5_luid_0x0_0x1_phys_0)\\Dedicated Usage\",\
\"\\\\PC\\GPU Process Memory(pid_5_luid_0x0_0x2_phys_0)\\Dedicated Usage\"\r\n\
\"t\",\"1048576\",\"2097152\"\r\n";
        let table = parse_typeperf(text).expect("valid csv");
        let pids: BTreeSet<u32> = [5].into_iter().collect();
        assert_eq!(gpu_memory(&table, &pids)[&5], vec![3.0]);
    }

    #[test]
    fn iso_utc_formats_known_instants() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(951_782_400 + 86_399), "2000-02-29T23:59:59Z");
        assert_eq!(iso_utc(1_795_363_200), "2026-11-22T16:00:00Z");
    }

    #[test]
    fn exe_profile_reads_the_cargo_folder() {
        assert_eq!(
            exe_profile(r"E:\P\Sucrose\target\release\sucrose.exe").as_deref(),
            Some("release")
        );
        assert_eq!(exe_profile("/x/target/debug/sucrose").as_deref(), Some("debug"));
        assert_eq!(exe_profile(r"C:\Program Files\Sucrose\Sucrose.Launcher.exe"), None);
    }

    fn report(side: &str, roles: &[&str], subtotal: bool) -> Report {
        let m = metrics(&[[point(100.0, 1.0, Some(5.0))].as_slice()]);
        Report {
            side: side.to_owned(),
            started: "2026-11-22T16:00:00Z".to_owned(),
            warmup_secs: 60,
            duration_secs: 300,
            logical_cpus: 8,
            build: Some("release".to_owned()),
            gpu_available: true,
            processes: roles
                .iter()
                .enumerate()
                .map(|(i, role)| ProcessReport {
                    pid: i as u32,
                    name: "x.exe".to_owned(),
                    role: (*role).to_owned(),
                    exe_profile: None,
                    metrics: m.clone(),
                    gpu_adapter: None,
                })
                .collect(),
            total: m.clone(),
            subtotal: subtotal.then(|| Subtotal {
                label: "MpvPlayer + Backgroundog".to_owned(),
                roles: strings(&CSHARP_SUBTOTAL_ROLES),
                metrics: m,
            }),
            dwm: None,
            warnings: Vec::new(),
        }
    }

    fn adapter(luid: &str, name: &str) -> AdapterRef {
        AdapterRef {
            luid: luid.to_owned(),
            name: name.to_owned(),
        }
    }

    /// A report whose processes of the given roles all ran on `adapter`.
    fn report_on(side: &str, roles: &[&str], on: Option<AdapterRef>) -> Report {
        let mut r = report(side, roles, side == "csharp");
        for p in &mut r.processes {
            p.gpu_adapter = on.clone();
        }
        r
    }

    #[test]
    fn row_has_both_totals_and_escapes_pipes() {
        let csharp = report("csharp", &["launcher", "mpv-player", "backgroundog"], true);
        let rust = report("rust", &["service", "renderer"], false);
        let row = format_row(&csharp, &rust, "scenario | one", "a\nb");
        assert!(row.starts_with("| 2026-11-22 | scenario / one | 3 processes: RAM 100 MB"));
        assert!(row.contains("; MpvPlayer + Backgroundog: RAM 100 MB"));
        assert!(row.contains("| service + 1 renderer: RAM 100 MB"));
        assert!(row.ends_with("| a b |"));
    }

    #[test]
    fn args_parse_both_modes_and_reject_mixes() {
        let sample = parse_args(&strings(&["--side", "rust", "--duration", "10"])).expect("sample args");
        assert!(matches!(
            sample,
            Task::Sample(SampleArgs {
                side: Side::Rust,
                duration: 10,
                warmup: 60,
                ..
            })
        ));
        let row = parse_args(&strings(&[
            "--row",
            "c.json",
            "r.json",
            "--scenario",
            "s",
            "--notes",
            "n",
        ]));
        assert!(matches!(row, Ok(Task::Row(_))));
        assert!(parse_args(&strings(&["--row", "c.json", "r.json", "--side", "rust"])).is_err());
        assert!(parse_args(&strings(&["--side", "go"])).is_err());
        assert!(parse_args(&strings(&["--side", "rust", "--duration", "0"])).is_err());
    }

    const TWO_ADAPTER_CSV: &str = "\"(PDH-CSV 4.0)\",\"\\\\PC\\GPU Engine(pid_10_luid_0x0_0xA_phys_0_eng_0_engtype_3D)\\Utilization Percentage\",\
\"\\\\PC\\GPU Engine(pid_10_luid_0x0_0xB_phys_0_eng_0_engtype_3D)\\Utilization Percentage\",\
\"\\\\PC\\GPU Engine(pid_20_luid_0x0_0xA_phys_0_eng_0_engtype_3D)\\Utilization Percentage\"\r\n\
\"t1\",\"1\",\"4\",\"0\"\r\n\
\"t2\",\"2\",\"5\",\"0\"\r\n";

    #[test]
    fn measure_adapter_of_pid_picks_the_busiest_luid() {
        let table = parse_typeperf(TWO_ADAPTER_CSV).expect("valid csv");
        let ten: BTreeSet<u32> = [10].into_iter().collect();
        assert_eq!(adapter_of_pids(&table, &ten).as_deref(), Some("0x0_0xB"));
        // pid 20 has an instance but never any utilization, pid 30 has no instance at all.
        let idle: BTreeSet<u32> = [20, 30].into_iter().collect();
        assert_eq!(adapter_of_pids(&table, &idle), None);
        let both: BTreeSet<u32> = [10, 20].into_iter().collect();
        assert_eq!(adapter_of_pids(&table, &both).as_deref(), Some("0x0_0xB"));
    }

    #[test]
    fn measure_adapter_ref_names_the_luid_or_says_unknown() {
        let list = [GpuAdapter {
            name: "NVIDIA GeForce RTX 3050 Laptop GPU".to_owned(),
            luid: "0x00000000_0x0000A1B2".to_owned(),
            dedicated_video_memory_mb: 4096,
        }];
        assert_eq!(
            adapter_ref("0x00000000_0x0000A1B2", &list),
            adapter("0x00000000_0x0000A1B2", "NVIDIA GeForce RTX 3050 Laptop GPU")
        );
        assert_eq!(adapter_ref("0x0_0x9", &list), adapter("0x0_0x9", UNKNOWN_ADAPTER));
    }

    #[test]
    fn measure_same_adapter_accepts_one_luid_and_refuses_the_rest() {
        let nvidia = adapter("0x0_0xA", "NVIDIA GeForce RTX 3050 Laptop GPU");
        let intel = adapter("0x0_0xB", "Intel(R) UHD Graphics");
        let csharp = report_on(
            "csharp",
            &["launcher", "mpv-player", "backgroundog"],
            Some(nvidia.clone()),
        );
        let rust = report_on("rust", &["service", "renderer", "renderer"], Some(nvidia.clone()));
        assert_eq!(same_adapter(&csharp, &rust), Ok(nvidia.clone()));

        let on_intel = report_on("rust", &["service", "renderer"], Some(intel.clone()));
        let err = same_adapter(&csharp, &on_intel).expect_err("different GPUs");
        assert!(err.contains("different GPUs"), "{err}");
        assert!(err.contains("C#: NVIDIA GeForce RTX 3050 Laptop GPU"), "{err}");
        assert!(err.contains("Rust: Intel(R) UHD Graphics"), "{err}");

        // One renderer on the other adapter is enough to refuse.
        let mut mixed = report_on("rust", &["service", "renderer", "renderer"], Some(nvidia.clone()));
        mixed.processes[2].gpu_adapter = Some(intel);
        assert!(same_adapter(&csharp, &mixed).is_err());

        // Same name, different LUID (a reboot between the runs) is refused with a hint.
        let rebooted = report_on("rust", &["service", "renderer"], Some(adapter("0x0_0xC", &nvidia.name)));
        let err = same_adapter(&csharp, &rebooted).expect_err("different LUIDs");
        assert!(err.contains("boot"), "{err}");
    }

    #[test]
    fn measure_same_adapter_refuses_a_missing_adapter_or_process() {
        let nvidia = adapter("0x0_0xA", "NVIDIA GeForce RTX 3050 Laptop GPU");
        let csharp = report_on("csharp", &["mpv-player"], Some(nvidia.clone()));
        let rust = report_on("rust", &["service", "renderer"], Some(nvidia.clone()));
        let no_adapter = report_on("rust", &["service", "renderer"], None);
        let err = same_adapter(&csharp, &no_adapter).expect_err("no adapter");
        assert!(err.contains("no GPU adapter"), "{err}");
        let old_csharp = report_on("csharp", &["mpv-player"], None);
        assert!(same_adapter(&old_csharp, &rust).is_err());
        let no_player = report_on("csharp", &["launcher"], Some(nvidia.clone()));
        assert!(same_adapter(&no_player, &rust).is_err());
        let no_renderer = report_on("rust", &["service"], Some(nvidia));
        assert!(same_adapter(&csharp, &no_renderer).is_err());
    }

    #[test]
    fn measure_report_without_adapter_fields_still_reads_and_is_refused() {
        let nvidia = adapter("0x0_0xA", "NVIDIA GeForce RTX 3050 Laptop GPU");
        let mut value =
            serde_json::to_value(report_on("csharp", &["mpv-player"], Some(nvidia.clone()))).expect("report encodes");
        value.as_object_mut().expect("object").remove("dwm");
        for p in value["processes"].as_array_mut().expect("processes") {
            p.as_object_mut().expect("process").remove("gpu_adapter");
        }
        let old: Report = serde_json::from_value(value).expect("an old result still reads");
        assert!(old.dwm.is_none());
        assert!(old.processes[0].gpu_adapter.is_none());
        let rust = report_on("rust", &["service", "renderer"], Some(nvidia));
        assert!(checked_row(&old, &rust, "s", "n").is_err());
    }

    #[test]
    fn measure_row_names_the_adapter_and_both_dwm_values() {
        let nvidia = adapter("0x0_0xA", "NVIDIA GeForce RTX 3050 Laptop GPU");
        let mut csharp = report_on("csharp", &["mpv-player"], Some(nvidia.clone()));
        let mut rust = report_on("rust", &["service", "renderer"], Some(nvidia.clone()));
        csharp.dwm = Some(DwmReport {
            gpu_avg_pct: 1.5,
            gpu_adapter: Some(nvidia.clone()),
        });
        rust.dwm = Some(DwmReport {
            gpu_avg_pct: 2.25,
            gpu_adapter: Some(nvidia),
        });
        let row = checked_row(&csharp, &rust, "scenario", "note.").expect("same adapter");
        assert!(
            row.contains("GPU: NVIDIA GeForce RTX 3050 Laptop GPU, both sides"),
            "{row}"
        );
        assert!(row.contains("dwm.exe GPU: C# 1.5 %, Rust 2.2 %"), "{row}");
        assert!(row.ends_with(" |"));
        // A note with or without a final full stop is separated from the GPU sentence the same way.
        assert!(row.contains("note. GPU:"), "{row}");
        let bare = checked_row(&csharp, &rust, "scenario", "no full stop").expect("same adapter");
        assert!(bare.contains("no full stop. GPU:"), "{bare}");
        let empty = checked_row(&csharp, &rust, "scenario", "").expect("same adapter");
        assert!(!empty.contains(". GPU:"), "{empty}");
    }

    #[test]
    fn measure_dwm_report_averages_the_compositor_and_names_its_adapter() {
        let table = parse_typeperf(TWO_ADAPTER_CSV).expect("valid csv");
        let adapters = [GpuAdapter {
            name: "Intel(R) UHD Graphics".to_owned(),
            luid: "0x0_0xB".to_owned(),
            dedicated_video_memory_mb: 128,
        }];
        let dwm_pids: BTreeSet<u32> = [10].into_iter().collect();
        let report = dwm_report(&table, &dwm_pids, &adapters).expect("dwm present");
        assert!((report.gpu_avg_pct - (4.0 + 5.0) / 2.0).abs() < 1e-9);
        assert_eq!(report.gpu_adapter, Some(adapter("0x0_0xB", "Intel(R) UHD Graphics")));
        assert!(dwm_report(&table, &BTreeSet::new(), &adapters).is_none());
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// 100 s sampled every 500 ms: a 20 s warm-up at 100 MB, three 20 s rounds that rise to a peak and fall back to an
    /// end value, and a 20 s idle window at 118 MB.
    fn rounds_series() -> Vec<(u64, f64)> {
        let peaks = [140.0, 150.0, 160.0];
        let ends = [130.0, 125.0, 120.0];
        let mut values: Vec<f64> = vec![100.0; 40];
        for round in 0..3 {
            for step in 1..=40u32 {
                let step = f64::from(step);
                values.push(if step <= 20.0 {
                    100.0 + (peaks[round] - 100.0) * step / 20.0
                } else {
                    peaks[round] + (ends[round] - peaks[round]) * (step - 20.0) / 20.0
                });
            }
        }
        values.extend(std::iter::repeat_n(118.0, 40));
        values
            .into_iter()
            .enumerate()
            .map(|(i, v)| ((i as u64 + 1) * 500, v))
            .collect()
    }

    fn series_of(points: &[(u64, f64)], build: Option<&str>) -> PortalSeries {
        PortalSeries {
            side: "portal".to_owned(),
            started: "2026-11-24T10:00:00Z".to_owned(),
            interval_ms: 500,
            pid: 7,
            build: build.map(str::to_owned),
            complete: true,
            ended_by: Some(ENDED_BY_TIME.to_owned()),
            samples: points
                .iter()
                .map(|(t, v)| PortalSample {
                    elapsed_ms: *t,
                    working_set_mb: *v,
                    private_mb: v * 2.0,
                })
                .collect(),
        }
    }

    #[test]
    fn portal_classify_picks_only_the_portal_executable() {
        let render = strings(&["sucrose.exe", "--render"]);
        assert_eq!(classify(Side::Portal, "sucrose-portal.exe", &[]), Some("portal"));
        assert_eq!(classify(Side::Portal, "Sucrose-Portal.EXE", &[]), Some("portal"));
        assert_eq!(classify(Side::Portal, "sucrose.exe", &render), None);
        assert_eq!(classify(Side::Portal, "Sucrose.Portal.exe", &[]), None);
    }

    #[test]
    fn portal_args_parse_the_three_modes_and_refuse_bad_values() {
        let sample = parse_args(&strings(&[
            "--side",
            "portal",
            "--seconds",
            "10",
            "--interval-ms",
            "500",
            "--out",
            "p.json",
        ]))
        .expect("portal sample args");
        assert!(matches!(
            sample,
            Task::Portal(PortalArgs {
                seconds: 10,
                interval_ms: 500,
                ..
            })
        ));
        let summary = parse_args(&strings(&[
            "--portal-summary",
            "p.json",
            "--warmup",
            "4",
            "--rounds",
            "1",
            "--idle",
            "4",
        ]))
        .expect("summary args");
        assert!(matches!(
            summary,
            Task::PortalSummary(PortalSummaryArgs {
                shape: PortalShape {
                    warmup: 4,
                    rounds: 1,
                    idle: 4
                },
                ..
            })
        ));
        let row = parse_args(&strings(&["--portal-row", "a.json", "b.json"])).expect("row args");
        assert!(matches!(
            row,
            Task::PortalRow(PortalRowArgs {
                shape: PortalShape {
                    warmup: 20,
                    rounds: 3,
                    idle: 20
                },
                ..
            })
        ));
        assert!(parse_args(&strings(&["--portal-summary", "p.json", "--rounds", "0"])).is_err());
        assert!(parse_args(&strings(&["--side", "portal", "--interval-ms", "0"])).is_err());
        assert!(parse_args(&strings(&["--side", "portal", "--seconds", "0"])).is_err());
        assert!(parse_args(&strings(&["--side", "portal", "--duration", "5"])).is_err());
        assert!(parse_args(&strings(&["--side", "rust", "--seconds", "5"])).is_err());
        assert!(parse_args(&strings(&["--side", "portal", "--portal-summary", "p.json"])).is_err());
    }

    #[test]
    fn measure_portal_series_gives_first_page_round_values_peak_and_final() {
        let summary = portal_summary(&rounds_series(), 20, 3, 20).expect("summary");
        assert!(approx(summary.first_page, 100.0), "{summary:?}");
        assert_eq!(summary.round_peak.len(), 3, "{summary:?}");
        assert_eq!(summary.round_end.len(), 3, "{summary:?}");
        for (got, want) in summary.round_peak.iter().zip([140.0, 150.0, 160.0]) {
            assert!(approx(*got, want), "{summary:?}");
        }
        for (got, want) in summary.round_end.iter().zip([130.0, 125.0, 120.0]) {
            assert!(approx(*got, want), "{summary:?}");
        }
        assert!(approx(summary.peak, 160.0), "{summary:?}");
        assert!(approx(summary.final_value, 118.0), "{summary:?}");
    }

    #[test]
    fn measure_portal_series_first_page_is_the_median_of_the_last_ten_seconds_of_the_warm_up() {
        let mut points = rounds_series();
        for point in points.iter_mut().take(20) {
            point.1 = 300.0;
        }
        let summary = portal_summary(&points, 20, 3, 20).expect("summary");
        assert!(approx(summary.first_page, 100.0), "{summary:?}");
        assert!(approx(summary.peak, 300.0), "{summary:?}");
    }

    #[test]
    fn measure_portal_series_refuses_a_series_without_room_or_samples() {
        let points = rounds_series();
        assert!(portal_summary(&points, 20, 0, 20).is_err());
        assert!(portal_summary(&[], 20, 3, 20).is_err());
        assert!(portal_summary(&points, 60, 3, 40).is_err());
        assert!(portal_summary(&points, 20, 3, 0).is_err());
        assert!(portal_summary(&points, 0, 3, 20).is_err());
    }

    #[test]
    fn portal_row_names_both_scenarios_and_the_three_values() {
        let real = series_of(&rounds_series(), Some("release"));
        let synth = series_of(&rounds_series(), Some("release"));
        let shape = PortalShape {
            warmup: 20,
            rounds: 3,
            idle: 20,
        };
        let row = portal_row(&real, &synth, &shape).expect("row");
        assert!(row.starts_with("| 2026-11-24 |"), "{row}");
        assert!(row.contains("real library"), "{row}");
        assert!(row.contains("500 synthetic"), "{row}");
        assert!(row.contains("first page 100 MB"), "{row}");
        assert!(row.contains("peak 160"), "{row}");
        assert!(row.contains("final 118"), "{row}");
        assert!(row.contains("private first page 200 MB"), "{row}");
        assert!(!row.contains('\n'), "{row}");
    }

    #[test]
    fn portal_row_refuses_a_series_that_is_not_a_release_build_or_not_complete() {
        let shape = PortalShape {
            warmup: 20,
            rounds: 3,
            idle: 20,
        };
        let release = series_of(&rounds_series(), Some("release"));
        let debug = series_of(&rounds_series(), Some("debug"));
        let unknown = series_of(&rounds_series(), None);
        assert!(portal_row(&release, &debug, &shape).is_err());
        assert!(portal_row(&unknown, &release, &shape).is_err());
        let mut cut = series_of(&rounds_series(), Some("release"));
        cut.complete = false;
        assert!(portal_row(&release, &cut, &shape).is_err());
    }

    const LOG_START: &str = "2026-11-24T10:00:00.000Z";
    const ROUND_STARTS_S: [u64; 3] = [20, 50, 80];
    const ROUND_ENDS_S: [u64; 3] = [40, 70, 100];

    /// Memory of the synthetic D-25 run: 100 MB on page 1, a ramp to 150 MB inside each round, then 130, 125 and 110 MB
    /// until the next round (the last value holds to the end).
    fn log_value(t_ms: u64) -> f64 {
        let t = t_ms as f64 / 1000.0;
        let settled = [130.0, 125.0, 110.0];
        for round in 0..3 {
            let (start, end) = (ROUND_STARTS_S[round] as f64, ROUND_ENDS_S[round] as f64);
            if t > start && t <= end {
                return 100.0 + 50.0 * (t - start) / (end - start);
            }
            let next = ROUND_STARTS_S.get(round + 1).map_or(f64::MAX, |s| *s as f64);
            if t > end && t <= next {
                return settled[round];
            }
        }
        100.0
    }

    fn log_points(seconds: u64) -> Vec<(u64, f64)> {
        (1..=seconds * 2).map(|i| (i * 500, log_value(i * 500))).collect()
    }

    /// The page events of the rounds above: page 2 at the start, page 3 5 s later, page 2 again, page 1 at the end.
    fn round_events(rounds: usize, first_s: u64) -> Vec<(u64, u32)> {
        let mut events = vec![(1, 1)];
        for round in 0..rounds {
            let start = if round == 0 { first_s } else { ROUND_STARTS_S[round] };
            let end = ROUND_ENDS_S[round];
            events.extend([(start, 2), (start + 5, 3), (start + 10, 2), (end, 1)]);
        }
        events
    }

    fn start_ms() -> i64 {
        parse_utc_ms(LOG_START).expect("a valid start")
    }

    fn page_log(events: &[(u64, u32)], pages: u32) -> String {
        let mut text = String::new();
        for (second, page) in events {
            let at = iso_utc_ms(u64::try_from(start_ms()).expect("positive") + second * 1000 + 250);
            let at = at.replace('Z', "123Z");
            let _ = writeln!(
                text,
                "{at}  DEBUG sucrose_portal::library: page shown page={page} pages={pages} thumbs=3 thumb_bytes=900 preview_playing=false"
            );
        }
        text
    }

    fn elapsed_events(events: &[(u64, u32)], pages: u32) -> Vec<PageEvent> {
        rebase_events(&page_events(&page_log(events, pages)), start_ms())
    }

    #[test]
    fn portal_log_page_events_keep_only_page_shown_lines_and_ignore_a_cut_last_line() {
        let log = "2026-11-24T10:00:01.500123Z  INFO sucrose_portal: starting\n\
                   2026-11-24T10:00:02.000000Z DEBUG sucrose_portal::library: page shown page=2 pages=17 thumbs=3\n\
                   2026-11-24T10:00:02.500000Z DEBUG other::module: page shown page=9 pages=17\n\
                   2026-11-24T10:00:03.000000Z DEBUG sucrose_portal::library: page shown thumbs=3\n\
                   2026-11-24T10:00:04.000000Z DEBUG sucrose_portal::library: page shown page=1 pages=17\n\
                   2026-11-24T10:00:05.000000Z DEBUG sucrose_portal::library: page shown page=1";
        let events = page_events(log);
        assert_eq!(events.len(), 2, "{events:?}");
        assert_eq!((events[0].page, events[0].pages), (2, Some(17)));
        assert_eq!((events[1].page, events[1].pages), (1, Some(17)));
        assert_eq!(events[1].at_ms - events[0].at_ms, 2000);
        assert_eq!(page_events("no newline at all").len(), 0);
    }

    #[test]
    fn portal_log_rounds_end_when_page_one_follows_the_last_page() {
        let pages = [1, 2, 3, 2, 1, 2, 3, 2, 1, 2, 3, 2, 1];
        let events: Vec<PageEvent> = pages
            .iter()
            .enumerate()
            .map(|(i, page)| PageEvent {
                at_ms: (i as i64 + 1) * 10_000,
                page: *page,
                pages: Some(3),
            })
            .collect();
        let (rounds, open) = split_rounds(&events);
        assert_eq!(open, None);
        // Event i is at (i + 1) * 10 s: round 1 runs from index 1 to index 4, round 2 from 5 to 8, round 3 from 9 to 12.
        let want = [(20_000, 50_000), (60_000, 90_000), (100_000, 130_000)];
        assert_eq!(rounds.len(), 3, "{rounds:?}");
        for (round, (start, end)) in rounds.iter().zip(want) {
            assert_eq!((round.start_ms, round.end_ms), (start, end));
        }
        // Paging that goes back before the last page is not a round end.
        let early: Vec<PageEvent> = [(1, 1), (2, 2), (3, 1), (4, 2)]
            .iter()
            .map(|(second, page)| PageEvent {
                at_ms: second * 1000,
                page: *page,
                pages: Some(3),
            })
            .collect();
        assert_eq!(split_rounds(&early), (Vec::new(), Some(2000)));
    }

    #[test]
    fn portal_log_summary_matches_a_synthetic_series() {
        let events = elapsed_events(&round_events(3, 20), 3);
        let summary = portal_log_summary(&log_points(130), &events, 3).expect("summary");
        let values = &summary.values;
        assert!(approx(values.first_page, 100.0), "{values:?}");
        for (got, want) in values.round_end.iter().zip([130.0, 125.0, 110.0]) {
            assert!(approx(*got, want), "{values:?}");
        }
        for got in &values.round_peak {
            assert!(approx(*got, 150.0), "{values:?}");
        }
        assert!(approx(values.peak, 150.0), "{values:?}");
        assert!(approx(values.final_value, 110.0), "{values:?}");
        assert_eq!(summary.rounds.len(), 3);
        // The log lines carry a 250 ms offset after the whole second.
        assert_eq!((summary.rounds[0].start_ms, summary.rounds[0].end_ms), (20_250, 40_250));
    }

    #[test]
    fn portal_log_summary_refuses_too_few_rounds_and_a_short_idle() {
        let two = elapsed_events(&round_events(2, 20), 3);
        let error = portal_log_summary(&log_points(130), &two, 3).expect_err("two rounds");
        assert!(
            error.contains("2 complete rounds") && error.contains("--rounds asks for 3"),
            "{error}"
        );

        let three = elapsed_events(&round_events(3, 20), 3);
        let error = portal_log_summary(&log_points(115), &three, 3).expect_err("15 s of idle");
        assert!(error.contains("14.8") && error.contains("at least 20 s"), "{error}");
        assert!(portal_log_summary(&log_points(121), &three, 3).is_ok());

        let short_warm_up = elapsed_events(&round_events(3, 5), 3);
        let error = portal_log_summary(&log_points(130), &short_warm_up, 3).expect_err("5 s warm-up");
        assert!(error.contains("warm-up") && error.contains("10 s"), "{error}");

        let error = portal_log_summary(&log_points(130), &three, 2).expect_err("more rounds than asked");
        assert!(error.contains("give --rounds 3"), "{error}");
        let mut paging_again = round_events(3, 20);
        paging_again.push((110, 2));
        let error =
            portal_log_summary(&log_points(130), &elapsed_events(&paging_again, 3), 3).expect_err("paging again");
        assert!(error.contains("paging again"), "{error}");
        assert!(portal_log_summary(&log_points(130), &[], 3).is_err());
    }

    #[test]
    fn portal_log_summary_is_the_same_twice() {
        let events = elapsed_events(&round_events(3, 20), 3);
        let first = portal_log_summary(&log_points(130), &events, 3).expect("summary");
        let second = portal_log_summary(&log_points(130), &events, 3).expect("summary");
        assert_eq!(first, second);
        let log = page_log(&round_events(3, 20), 3);
        let mut series = series_of(&log_points(130), Some("release"));
        series.started = LOG_START.to_owned();
        let (a, _) = log_summaries(&series, &log, 3).expect("summary");
        let (b, _) = log_summaries(&series, &log, 3).expect("summary");
        assert_eq!(a, b);
    }

    #[test]
    fn portal_log_row_says_the_rounds_come_from_the_page_log() {
        let log = page_log(&round_events(3, 20), 3);
        let mut real = series_of(&log_points(130), Some("release"));
        real.started = LOG_START.to_owned();
        real.complete = false;
        real.ended_by = Some(ENDED_BY_EXIT.to_owned());
        let mut synth = real.clone();
        synth.complete = true;
        synth.ended_by = Some(ENDED_BY_TIME.to_owned());
        let row = portal_row_from_logs((&real, &log), (&synth, &log), 3).expect("row");
        assert!(row.starts_with("| 2026-11-24 |"), "{row}");
        assert!(row.contains("the rounds come from the Portal's page log"), "{row}");
        assert!(row.contains("first page 100 MB") && row.contains("final 110"), "{row}");
        assert!(!row.contains('\n'), "{row}");
        let mut old = real.clone();
        old.ended_by = None;
        assert!(portal_row_from_logs((&old, &log), (&synth, &log), 3).is_err());
        let mut debug = synth.clone();
        debug.build = Some("debug".to_owned());
        assert!(portal_row_from_logs((&real, &log), (&debug, &log), 3).is_err());
        assert!(portal_row_from_logs((&real, "nothing"), (&synth, &log), 3).is_err());
    }

    #[test]
    fn portal_series_start_keeps_milliseconds_and_old_files_still_parse() {
        let text = iso_utc_ms(1_795_514_400_123);
        assert_eq!(text, "2026-11-24T10:00:00.123Z");
        assert_eq!(parse_utc_ms(&text), Some(1_795_514_400_123));
        // Whole seconds (older series) and microseconds (the log) parse too; other shapes do not.
        assert_eq!(parse_utc_ms("2026-11-24T10:00:00Z"), Some(1_795_514_400_000));
        assert_eq!(parse_utc_ms("2026-11-24T10:00:00.123456Z"), Some(1_795_514_400_123));
        assert_eq!(parse_utc_ms("2026-11-24T10:00:00.5Z"), Some(1_795_514_400_500));
        assert_eq!(parse_utc_ms("2026-11-24 10:00:00Z"), None);
        assert_eq!(parse_utc_ms("2026-11-24T10:00:00+01:00"), None);
        assert_eq!(parse_utc_ms("2026-13-24T10:00:00Z"), None);
        assert_eq!(parse_utc_ms("1970-01-01T00:00:00.000Z"), Some(0));
        let old = r#"{"side":"portal","started":"2026-11-24T12:19:24Z","interval_ms":500,"pid":1,"build":"release","complete":true,"samples":[]}"#;
        let series: PortalSeries = serde_json::from_str(old).expect("an older series parses");
        assert_eq!(series.ended_by, None);
        assert_eq!(parse_utc_ms(&series.started), Some(1_795_522_764_000));
        let new = serde_json::to_string(&series_of(&[(500, 1.0)], Some("release"))).expect("encodes");
        assert!(new.contains("\"ended_by\":\"time\""), "{new}");
    }

    #[test]
    fn portal_log_clock_text_and_offsets() {
        assert_eq!(parse_utc_offset("+0300"), Some(180));
        assert_eq!(parse_utc_offset("-05:30"), Some(-330));
        assert_eq!(parse_utc_offset("180\r\n"), Some(180));
        assert_eq!(parse_utc_offset("-60"), Some(-60));
        assert_eq!(parse_utc_offset("nope"), None);
        let ms = 1_795_514_400_123;
        assert_eq!(clock_text(ms, 0), "10:00:00.1");
        assert_eq!(clock_text(ms, 180), "13:00:00.1");
        assert_eq!(clock_text(ms, -600), "00:00:00.1");
        assert_eq!(offset_label(Some(180)), "UTC+03:00");
        assert_eq!(offset_label(Some(-330)), "UTC-05:30");
        assert_eq!(offset_label(Some(0)), "UTC");
        let spans = [RoundSpan {
            start_ms: 20_000,
            end_ms: 40_500,
        }];
        let lines = round_times(&spans, 1_795_514_400_000, Some(180));
        assert_eq!(lines, vec!["round 1: 13:00:20.0 to 13:00:40.5 (20.5 s)".to_owned()]);
    }

    #[test]
    fn portal_log_args_choose_the_log_split_and_refuse_fixed_window_flags() {
        let summary = parse_args(&strings(&[
            "--portal-summary",
            "p.json",
            "--portal-log",
            "p.log",
            "--rounds",
            "3",
        ]))
        .expect("log summary args");
        assert!(matches!(
            summary,
            Task::PortalSummary(PortalSummaryArgs {
                log: Some(_),
                shape: PortalShape { rounds: 3, .. },
                ..
            })
        ));
        let row = parse_args(&strings(&[
            "--portal-row",
            "a.json",
            "b.json",
            "--portal-logs",
            "a.log",
            "b.log",
        ]))
        .expect("log row args");
        assert!(matches!(row, Task::PortalRow(PortalRowArgs { logs: Some(_), .. })));
        assert!(
            parse_args(&strings(&[
                "--portal-summary",
                "p.json",
                "--portal-log",
                "p.log",
                "--warmup",
                "5"
            ]))
            .is_err()
        );
        assert!(
            parse_args(&strings(&[
                "--portal-summary",
                "p.json",
                "--portal-log",
                "p.log",
                "--idle",
                "5"
            ]))
            .is_err()
        );
        assert!(parse_args(&strings(&["--portal-log", "p.log"])).is_err());
        assert!(parse_args(&strings(&["--portal-logs", "a.log", "b.log"])).is_err());
        let plain = parse_args(&strings(&["--portal-summary", "p.json"])).expect("fixed windows still work");
        assert!(matches!(
            plain,
            Task::PortalSummary(PortalSummaryArgs { log: None, .. })
        ));
    }
}
