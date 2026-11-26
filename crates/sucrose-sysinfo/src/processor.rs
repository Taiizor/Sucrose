//! The Processor group: the legacy `GetProcessorInfo` object. The static fields (name, core and thread counts) come from
//! WMI once; the load of the whole processor and of each logical processor comes from the PDH counter
//! `Processor\% Processor Time`, read once a second, with the running minimum and maximum the legacy code keeps.

use serde_json::{Value, json};

use crate::{SysInfoError, float, wmi_info::ProcessorInfo};

/// The legacy `Type` of every core entry: `PerformanceCounterType.Timer100NsInverse`, the type of the processor time
/// counter, which Newtonsoft writes as its number.
const COUNTER_TYPE: i64 = 557_909_248;
/// The text .NET gives that counter type (`$"{CounterType}"`).
const COUNTER_TYPE_TEXT: &str = "Timer100NsInverse";

/// Keeps the figures between samples and writes the legacy object. The load values are fed in; where they come from is
/// the sampler's business.
#[derive(Debug, Clone)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct Tracker {
    info: ProcessorInfo,
    /// Whether the static fields were read, which is all the legacy `State` means.
    state: bool,
    count: usize,
    counter_type: &'static str,
    now: f32,
    max: f32,
    min: f32,
    /// The latest load of each core by its index, for a core whose counter gives nothing this time.
    last_cores: Vec<(usize, f32)>,
    cores: Vec<Value>,
    cores_now: f32,
    cores_max: f32,
    cores_min: f32,
}

#[cfg_attr(not(windows), allow(dead_code))]
impl Tracker {
    /// The legacy initial figures: nothing sampled, the minimums at 100.
    pub(crate) fn new(info: Option<ProcessorInfo>, count: usize) -> Self {
        Self {
            state: info.is_some(),
            info: info.unwrap_or_default(),
            count,
            counter_type: "",
            now: 0.0,
            max: 0.0,
            min: 100.0,
            last_cores: Vec::new(),
            cores: Vec::new(),
            cores_now: 0.0,
            cores_max: 0.0,
            cores_min: 100.0,
        }
    }

    pub(crate) fn now(&self) -> f32 {
        self.now
    }

    /// The latest load of core `index`, 0 before the first sample.
    pub(crate) fn last_core(&self, index: usize) -> f32 {
        self.last_cores
            .iter()
            .find(|(core, _)| *core == index)
            .map_or(0.0, |(_, load)| *load)
    }

    /// Takes one sample: the load of the whole processor and of each core (index, load), in percent.
    ///
    /// The figures follow the legacy code, quirks included: `Max`/`Min` are the running extremes of `Now`; `CoresNow` is the
    /// highest core; `CoresMax`/`CoresMin` are updated inside the per-core loop with the highest load seen so far in it,
    /// so `CoresMin` follows the first cores rather than the lowest one.
    pub(crate) fn record(&mut self, now: f32, cores: &[(usize, f32)]) {
        self.counter_type = COUNTER_TYPE_TEXT;
        self.now = now.clamp(0.0, 100.0);
        self.max = self.now.max(self.max).clamp(0.0, 100.0);
        self.min = self.now.min(self.min).clamp(0.0, 100.0);

        self.cores_now = 0.0;
        let loads: Vec<(usize, f32)> = cores
            .iter()
            .map(|(index, load)| (*index, load.clamp(0.0, 100.0)))
            .collect();
        for (_, load) in &loads {
            self.cores_now = load.max(self.cores_now).clamp(0.0, 100.0);
            self.cores_max = self.cores_now.max(self.cores_max).clamp(0.0, 100.0);
            self.cores_min = self.cores_now.min(self.cores_min).clamp(0.0, 100.0);
        }
        // `UpdateExtremes` of the legacy code: the highest and the lowest entries are flagged.
        let highest = loads.iter().map(|(_, load)| *load).fold(f32::MIN, f32::max);
        let lowest = loads.iter().map(|(_, load)| *load).fold(f32::MAX, f32::min);
        self.cores = loads
            .iter()
            .map(|(index, load)| {
                json!({
                    "Index": index,
                    "IsMax": *load >= highest,
                    "IsMin": *load <= lowest,
                    "Now": float(*load),
                    "Name": format!("Core #{index}"),
                    "TypeText": COUNTER_TYPE_TEXT,
                    "Type": COUNTER_TYPE
                })
            })
            .collect();
        self.last_cores = loads;
    }

    /// The legacy object: the keys in the order of `Data.cs`.
    pub(crate) fn payload(&self) -> Value {
        json!({
            "Max": float(self.max),
            "Min": float(self.min),
            "Now": float(self.now),
            "Core": self.info.cores,
            "Name": self.info.name,
            "Type": self.counter_type,
            "Cores": self.cores,
            "State": self.state,
            "Thread": self.info.threads,
            "CoresMax": float(self.cores_max),
            "CoresMin": float(self.cores_min),
            "CoresNow": float(self.cores_now),
            "ProcessorCount": self.count
        })
    }
}

/// The Processor group of the legacy default, which a page sees while the counters cannot be read: `State` false, empty
/// text, nothing sampled, the minimums at 100.
#[must_use]
pub fn processor_unavailable() -> Value {
    Tracker::new(None, 0).payload()
}

#[cfg(windows)]
mod pdh {
    //! A small safe layer over the PDH query API: one query, counters added by their English path (so the names do not
    //! depend on the display language, as the .NET `PerformanceCounter` names do not), closed on drop.

    use windows::{
        Win32::System::Performance::{
            PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PDH_HCOUNTER,
            PDH_HQUERY, PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterValue,
            PdhOpenQueryW,
        },
        core::{HSTRING, PCWSTR},
    };

    use crate::SysInfoError;

    /// `ERROR_SUCCESS` as the PDH functions return it.
    const SUCCESS: u32 = 0;

    fn failed(call: &'static str, status: u32) -> SysInfoError {
        SysInfoError::Os {
            call,
            message: format!("PDH status {status:#010x}"),
        }
    }

    /// An open PDH query; the counters added to it die with it.
    pub(super) struct Query(PDH_HQUERY);

    // SAFETY: a PDH query handle is not tied to the thread that opened it; it only must not be used by two threads at
    // once, and `Query` is not `Sync` and every use takes `&self` or `&mut` of its single owner.
    unsafe impl Send for Query {}

    /// A counter of a [`Query`]; valid while that query is open.
    #[derive(Clone, Copy)]
    pub(super) struct Counter(PDH_HCOUNTER);

    // SAFETY: a counter handle is an opaque id of the query it was added to; it is only used through that query, which
    // is itself used by one thread at a time (see `Query`).
    unsafe impl Send for Counter {}

    impl Query {
        pub(super) fn open() -> Result<Self, SysInfoError> {
            let mut handle = PDH_HQUERY::default();
            // SAFETY: a null data source means the live counters; `handle` is a valid out pointer for the call.
            let status = unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut handle) };
            if status == SUCCESS {
                Ok(Self(handle))
            } else {
                Err(failed("PdhOpenQuery", status))
            }
        }

        pub(super) fn add(&self, path: &str) -> Result<Counter, SysInfoError> {
            let mut counter = PDH_HCOUNTER::default();
            let path = HSTRING::from(path);
            // SAFETY: `self.0` is an open query, `path` is a NUL-terminated wide string that outlives the call and
            // `counter` is a valid out pointer.
            let status = unsafe { PdhAddEnglishCounterW(self.0, &path, 0, &mut counter) };
            if status == SUCCESS {
                Ok(Counter(counter))
            } else {
                Err(failed("PdhAddEnglishCounter", status))
            }
        }

        /// Takes a sample of every counter of the query. Rate counters need two samples before they give a value.
        pub(super) fn collect(&self) -> Result<(), SysInfoError> {
            // SAFETY: `self.0` is an open query.
            let status = unsafe { PdhCollectQueryData(self.0) };
            if status == SUCCESS {
                Ok(())
            } else {
                Err(failed("PdhCollectQueryData", status))
            }
        }

        /// The value of `counter` between the last two samples, `None` when PDH has no valid value for it.
        pub(super) fn value(&self, counter: Counter) -> Option<f64> {
            let mut value = PDH_FMT_COUNTERVALUE::default();
            // SAFETY: `counter` belongs to this open query and `value` is a valid out pointer.
            let status = unsafe { PdhGetFormattedCounterValue(counter.0, PDH_FMT_DOUBLE, None, &mut value) };
            if status != SUCCESS || !matches!(value.CStatus, PDH_CSTATUS_VALID_DATA | PDH_CSTATUS_NEW_DATA) {
                return None;
            }
            // SAFETY: the call succeeded with PDH_FMT_DOUBLE, so the double member is the one that was written.
            Some(unsafe { value.Anonymous.doubleValue })
        }
    }

    impl Drop for Query {
        fn drop(&mut self) {
            // SAFETY: `self.0` is an open query that nothing uses after this; closing it releases its counters. The
            // status is ignored: nothing can be done about a failing close.
            let _ = unsafe { PdhCloseQuery(self.0) };
        }
    }
}

/// Reads the processor load. It opens PDH counters and keeps the running figures, so it exists only while a page needs
/// the Processor group (D-24) and closes its query when dropped.
#[cfg(windows)]
pub struct ProcessorSampler {
    query: pdh::Query,
    total: pdh::Counter,
    cores: Vec<(usize, pdh::Counter)>,
    tracker: Tracker,
    /// When the query last collected; rate counters give nothing useful for two collections a few milliseconds apart.
    collected: std::time::Instant,
    /// Why the static fields (WMI) are missing, until a caller has taken it with [`ProcessorSampler::info_error`].
    info_error: Option<SysInfoError>,
}

#[cfg(windows)]
impl ProcessorSampler {
    /// The shortest time between two real samples.
    const MIN_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

    /// Opens the counters (the whole processor and one per logical processor), takes the first sample and reads the
    /// static fields from WMI. A core whose counter PDH does not know (a machine with more than 64 logical processors
    /// names its instances differently) is left out of `Cores`; the group stays usable.
    ///
    /// # Errors
    /// [`SysInfoError::Os`] when PDH cannot open the query or does not know the counter of the whole processor. A WMI
    /// failure is not an error here: the group then carries `State` false, see [`ProcessorSampler::info_error`].
    pub fn new() -> Result<Self, SysInfoError> {
        let count = std::thread::available_parallelism().map_or(0, std::num::NonZeroUsize::get);
        let query = pdh::Query::open()?;
        let total = query.add(r"\Processor(_Total)\% Processor Time")?;
        let cores: Vec<(usize, pdh::Counter)> = (0..count)
            .filter_map(|index| {
                query
                    .add(&format!(r"\Processor({index})\% Processor Time"))
                    .ok()
                    .map(|counter| (index, counter))
            })
            .collect();
        // The first collection only primes the rate counters, as the first `NextValue()` of the legacy code does.
        query.collect()?;
        let collected = std::time::Instant::now();
        let (info, info_error) = match crate::wmi_info::processor_info() {
            Ok(info) => (Some(info), None),
            Err(error) => (None, Some(error)),
        };
        Ok(Self {
            query,
            total,
            cores,
            tracker: Tracker::new(info, count),
            collected,
            info_error,
        })
    }

    /// The reason the static fields could not be read, once; `None` when they were read or it was taken already.
    pub fn info_error(&mut self) -> Option<SysInfoError> {
        self.info_error.take()
    }

    /// Takes a sample and returns the group. Called again within a quarter of a second, or when PDH gives no value, it
    /// keeps the figures it has.
    pub fn sample(&mut self) -> Value {
        if self.collected.elapsed() >= Self::MIN_INTERVAL && self.query.collect().is_ok() {
            self.collected = std::time::Instant::now();
            let now = self
                .query
                .value(self.total)
                .map_or_else(|| self.tracker.now(), |load| load as f32);
            let cores: Vec<(usize, f32)> = self
                .cores
                .iter()
                .map(|(index, counter)| {
                    let load = self
                        .query
                        .value(*counter)
                        .map_or_else(|| self.tracker.last_core(*index), |load| load as f32);
                    (*index, load)
                })
                .collect();
            self.tracker.record(now, &cores);
        }
        self.tracker.payload()
    }
}

/// Reads the processor load (not available on this OS).
#[cfg(not(windows))]
pub struct ProcessorSampler {
    _private: (),
}

#[cfg(not(windows))]
impl ProcessorSampler {
    /// Opens the counters.
    ///
    /// # Errors
    /// [`SysInfoError::Unsupported`]: there is no provider on this OS.
    pub fn new() -> Result<Self, SysInfoError> {
        Err(SysInfoError::Unsupported("the processor counters"))
    }

    /// The reason the static fields could not be read; never set here.
    pub fn info_error(&mut self) -> Option<SysInfoError> {
        None
    }

    /// The group with the latest figures.
    pub fn sample(&mut self) -> Value {
        processor_unavailable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> ProcessorInfo {
        ProcessorInfo {
            name: "Test CPU".to_owned(),
            cores: 4,
            threads: 8,
        }
    }

    #[test]
    fn the_running_extremes_follow_the_total_load() {
        let mut tracker = Tracker::new(Some(info()), 2);

        tracker.record(40.0, &[(0, 10.0), (1, 70.0)]);
        tracker.record(10.0, &[(0, 5.0), (1, 15.0)]);
        tracker.record(25.0, &[(0, 25.0), (1, 25.0)]);

        let value = tracker.payload();
        assert_eq!(value["Now"].as_f64(), Some(25.0));
        assert_eq!(value["Max"].as_f64(), Some(40.0));
        assert_eq!(value["Min"].as_f64(), Some(10.0));
        assert_eq!(value["State"], true);
        assert_eq!(value["Name"], "Test CPU");
        assert_eq!(value["Core"], 4);
        assert_eq!(value["Thread"], 8);
        assert_eq!(value["Type"], "Timer100NsInverse");
    }

    #[test]
    fn the_cores_flag_the_highest_and_the_lowest_and_the_cores_now_is_the_highest() {
        let mut tracker = Tracker::new(Some(info()), 3);

        tracker.record(30.0, &[(0, 20.0), (1, 90.0), (2, 5.0)]);

        let value = tracker.payload();
        let cores = value["Cores"].as_array().expect("an array");
        assert_eq!(cores.len(), 3);
        assert_eq!(cores[1]["IsMax"], true);
        assert_eq!(cores[1]["IsMin"], false);
        assert_eq!(cores[2]["IsMin"], true);
        assert_eq!(cores[0]["IsMax"], false);
        assert_eq!(cores[2]["Name"], "Core #2");
        assert_eq!(value["CoresNow"].as_f64(), Some(90.0));
    }

    #[test]
    fn the_cores_min_and_max_are_updated_inside_the_core_loop_as_the_legacy_code_does() {
        let mut tracker = Tracker::new(Some(info()), 3);

        tracker.record(30.0, &[(0, 20.0), (1, 90.0), (2, 5.0)]);

        let value = tracker.payload();
        assert_eq!(value["CoresMax"].as_f64(), Some(90.0));
        assert_eq!(
            value["CoresMin"].as_f64(),
            Some(20.0),
            "the running highest after the first core, not the lowest core"
        );
    }

    #[test]
    fn loads_are_clamped_to_a_percentage() {
        let mut tracker = Tracker::new(Some(info()), 1);

        tracker.record(130.0, &[(0, -4.0)]);

        let value = tracker.payload();
        assert_eq!(value["Now"].as_f64(), Some(100.0));
        assert_eq!(value["Cores"][0]["Now"].as_f64(), Some(0.0));
    }

    #[test]
    fn missing_static_fields_give_state_false_and_the_load_keeps_flowing() {
        let mut tracker = Tracker::new(None, 4);

        tracker.record(12.0, &[]);

        let value = tracker.payload();
        assert_eq!(value["State"], false);
        assert_eq!(value["Name"], "");
        assert_eq!(value["Now"].as_f64(), Some(12.0));
        assert_eq!(value["ProcessorCount"], 4);
    }
}
