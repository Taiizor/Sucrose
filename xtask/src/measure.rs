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

  --side      rust: every sucrose.exe (service and --render renderers); csharp: Sucrose.Launcher,
              Sucrose.Live.MpvPlayer and Sucrose.Backgroundog
  --warmup    seconds to wait before sampling (default 60)
  --duration  seconds to sample, once per second (default 300)
  --out       result file (default target/measure/<side>.json)
  --row       print the docs/MEASUREMENTS.md table row for two result files
";

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
}

impl Side {
    fn parse(text: &str) -> Result<Self, String> {
        match text {
            "rust" => Ok(Self::Rust),
            "csharp" => Ok(Self::Csharp),
            other => Err(format!("--side must be rust or csharp, not '{other}'")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Csharp => "csharp",
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

#[derive(Debug)]
enum Task {
    Sample(SampleArgs),
    Row(RowArgs),
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

fn parse_args(args: &[String]) -> Result<Task, String> {
    let mut side = None;
    let mut warmup = DEFAULT_WARMUP;
    let mut duration = DEFAULT_DURATION;
    let mut out = None;
    let mut row: Option<(String, String)> = None;
    let mut scenario = None;
    let mut notes = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--side" => side = Some(Side::parse(value(&mut it, "--side")?)?),
            "--warmup" => warmup = seconds(value(&mut it, "--warmup")?, "--warmup")?,
            "--duration" => duration = seconds(value(&mut it, "--duration")?, "--duration")?,
            "--out" => out = Some(value(&mut it, "--out")?.to_owned()),
            "--row" => {
                let csharp = value(&mut it, "--row")?.to_owned();
                let rust = value(&mut it, "--row")?.to_owned();
                row = Some((csharp, rust));
            }
            "--scenario" => scenario = Some(value(&mut it, "--scenario")?.to_owned()),
            "--notes" => notes = Some(value(&mut it, "--notes")?.to_owned()),
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    match (row, side) {
        (Some(_), Some(_)) => Err("--row and --side cannot be combined".to_owned()),
        (Some((csharp, rust)), None) => Ok(Task::Row(RowArgs {
            csharp,
            rust,
            scenario: scenario.ok_or("--row needs --scenario")?,
            notes: notes.ok_or("--row needs --notes")?,
        })),
        (None, Some(side)) => {
            if duration == 0 {
                return Err("--duration must be at least 1".to_owned());
            }
            Ok(Task::Sample(SampleArgs {
                side,
                warmup,
                duration,
                out,
            }))
        }
        (None, None) => Err("give --side or --row".to_owned()),
    }
}

/// `cargo xtask measure ...`
pub fn run(args: &[String]) -> ExitCode {
    let result = parse_args(args).and_then(|task| match task {
        Task::Sample(sample_args) => sample(&sample_args),
        Task::Row(row_args) => print_row(&row_args),
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
        Side::Rust => None,
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
    let notes = format!(
        "{notes} GPU: {}, both sides. dwm.exe GPU: C# {}, Rust {}.",
        adapter.name,
        dwm_text(csharp),
        dwm_text(rust)
    );
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
}
