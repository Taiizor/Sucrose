//! The on-demand data hub (D-24, D-008): the service module that samples system data for web wallpapers. A renderer
//! whose page needs data says which groups (`DataWanted`); the hub exists only while at least one running renderer
//! wants a group, samples only the wanted groups (one collector per group, whoever else wants it), and ends when the
//! last such renderer does. It never reads the performance-counter setting: web data no longer follows it.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, bounded};
use serde_json::Value;
use sucrose_core::compatible::DataGroup;
use sucrose_platform::PlatformError;
use sucrose_sysinfo::{AudioCapture, MediaSession, ProcessorSampler, SysInfoError};

use crate::modules::{module_off, start_module};

/// The hub's tick: the legacy Backgroundog timer, which refreshes the Date group every time.
pub const TICK: Duration = Duration::from_millis(100);

/// How a snapshot offered to a renderer ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Queued for the renderer.
    Sent,
    /// The renderer's queue is full: the snapshot is dropped and the next tick sends a fresh one, so the hub never
    /// waits for a renderer that is behind (T-04-24).
    Dropped,
    /// The renderer has no connection (any more).
    Gone,
}

/// Reads one group; `None` when this build has no provider for it or the OS gave nothing. The text is the group's JSON.
pub type Sampler = Arc<dyn Fn(DataGroup) -> Option<String> + Send + Sync>;

/// Offers the snapshots of one tick to the renderer of a slot.
pub type Deliver = Arc<dyn Fn(u8, Vec<(DataGroup, String)>) -> Delivery + Send + Sync>;

/// Tells the sampler that no running page wants `group` any more, so it can let go of what it holds for it (an open
/// performance-counter query, a cached read).
pub type Release = Arc<dyn Fn(DataGroup) + Send + Sync>;

/// How many ticks pass between two readings of `group`: the legacy rates (Date every tick, the performance groups once
/// a second, the battery every two seconds). Bios and Motherboard are read from WMI once and kept by the sampler, so
/// asking for them every second costs a copy of a cached text; asking less often would make a page that starts later
/// wait for them.
#[must_use]
pub fn every_ticks(group: DataGroup) -> u64 {
    match group {
        DataGroup::Date | DataGroup::Audio => 1,
        DataGroup::Memory
        | DataGroup::Processor
        | DataGroup::Graphic
        | DataGroup::Network
        | DataGroup::Storage
        | DataGroup::Bios
        | DataGroup::Motherboard => 10,
        DataGroup::Battery => 20,
    }
}

/// What one renderer asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Wanted {
    /// The process that asked; an exit of an older process of the same slot does not remove a newer entry.
    pid: u32,
    groups: Vec<DataGroup>,
}

type Shared = Arc<Mutex<BTreeMap<u8, Wanted>>>;

fn lock(wanted: &Shared) -> MutexGuard<'_, BTreeMap<u8, Wanted>> {
    wanted.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The running hub: a worker thread that ticks until the hub is dropped.
pub struct DataHub {
    /// Dropping it ends the worker's wait.
    stop: Option<Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

impl DataHub {
    /// Starts the worker.
    fn start(wanted: Shared, sample: Sampler, deliver: Deliver) -> std::io::Result<Self> {
        let (stop, stopped) = bounded::<()>(0);
        let worker = thread::Builder::new()
            .name("data-hub".to_owned())
            .spawn(move || run(&wanted, &stopped, &sample, &deliver))?;
        tracing::info!("data hub started");
        Ok(Self {
            stop: Some(stop),
            worker: Some(worker),
        })
    }
}

impl Drop for DataHub {
    fn drop(&mut self) {
        // Closing the channel wakes the worker at once; the join ends the thread before the module is gone.
        drop(self.stop.take());
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            tracing::error!("the data hub worker panicked");
        }
        tracing::info!("data hub stopped");
    }
}

/// The worker: one pass per [`TICK`] until the stop channel closes.
fn run(wanted: &Shared, stopped: &Receiver<()>, sample: &Sampler, deliver: &Deliver) {
    let mut tick: u64 = 0;
    let mut missing = HashSet::new();
    let mut next = Instant::now() + TICK;
    loop {
        match stopped.recv_timeout(next.saturating_duration_since(Instant::now())) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
        }
        // A slow pass skips the ticks it missed instead of catching up in a burst.
        next = (next + TICK).max(Instant::now());
        let slots: Vec<(u8, Vec<DataGroup>)> = lock(wanted)
            .iter()
            .map(|(slot, entry)| (*slot, entry.groups.clone()))
            .collect();
        pass(tick, &slots, sample, deliver, &mut missing);
        tick = tick.wrapping_add(1);
    }
}

/// One pass: reads each group some slot wants and that is due this tick once, then offers every slot the snapshots of
/// its own groups, slots in order, so each renderer sees the snapshots in the order they were taken.
fn pass(
    tick: u64,
    slots: &[(u8, Vec<DataGroup>)],
    sample: &Sampler,
    deliver: &Deliver,
    missing: &mut HashSet<DataGroup>,
) {
    let mut snapshots: HashMap<DataGroup, String> = HashMap::new();
    for group in DataGroup::ALL {
        let wanted = slots.iter().any(|(_, groups)| groups.contains(&group));
        if !wanted || !tick.is_multiple_of(every_ticks(group)) {
            continue;
        }
        match sample(group) {
            Some(text) => {
                snapshots.insert(group, text);
            }
            None => {
                if missing.insert(group) {
                    tracing::debug!(?group, "no data for this group on this build, it is skipped");
                }
            }
        }
    }
    for (slot, groups) in slots {
        let batch: Vec<(DataGroup, String)> = DataGroup::ALL
            .into_iter()
            .filter(|group| groups.contains(group))
            .filter_map(|group| snapshots.get(&group).map(|text| (group, text.clone())))
            .collect();
        if batch.is_empty() {
            continue;
        }
        match deliver(*slot, batch) {
            Delivery::Sent => {}
            Delivery::Dropped => tracing::debug!(slot, "a renderer is behind, this data snapshot is dropped"),
            Delivery::Gone => tracing::debug!(slot, "a renderer has no connection, its data snapshot is dropped"),
        }
    }
}

/// The service's side of the hub: who wants what, and the hub that exists exactly while somebody does.
pub struct DataModule {
    wanted: Shared,
    hub: Option<DataHub>,
    sample: Sampler,
    deliver: Deliver,
    /// Called for a group when the last page that wanted it is gone, and for every group when the hub stops.
    release: Option<Release>,
    /// The groups some renderer wanted at the last change, to see which ones were dropped.
    held: HashSet<DataGroup>,
}

impl DataModule {
    /// A module with no hub: nothing runs until a renderer wants data.
    pub fn new(sample: Sampler, deliver: Deliver) -> Self {
        module_off("data hub");
        Self {
            wanted: Shared::default(),
            hub: None,
            sample,
            deliver,
            release: None,
            held: HashSet::new(),
        }
    }

    /// Sets what is told when a group is no longer wanted.
    #[must_use]
    pub fn with_release(mut self, release: Release) -> Self {
        self.release = Some(release);
        self
    }

    /// The renderer `pid` of `slot` needs `groups` (an empty list needs nothing). Starts the hub when it is the first.
    pub fn want(&mut self, slot: u8, pid: u32, groups: &[DataGroup]) {
        let mut unique: Vec<DataGroup> = Vec::new();
        for group in groups {
            if !unique.contains(group) {
                unique.push(*group);
            }
        }
        if unique.is_empty() {
            self.release(slot, Some(pid));
            return;
        }
        tracing::debug!(slot, pid, groups = ?unique, "a renderer wants data");
        lock(&self.wanted).insert(slot, Wanted { pid, groups: unique });
        self.follow();
    }

    /// The renderer of `slot` ended or reconnected: its groups are no longer wanted. With `pid` only the entry of that
    /// process goes, so the exit of a replaced renderer leaves its replacement alone. Stops the hub with the last one.
    pub fn release(&mut self, slot: u8, pid: Option<u32>) {
        {
            let mut wanted = lock(&self.wanted);
            if wanted
                .get(&slot)
                .is_some_and(|entry| pid.is_none_or(|pid| entry.pid == pid))
            {
                wanted.remove(&slot);
            }
        }
        self.follow();
    }

    /// Whether the hub exists.
    #[cfg(test)]
    #[must_use]
    pub fn running(&self) -> bool {
        self.hub.is_some()
    }

    /// Starts or stops the hub to match whether any renderer wants data.
    fn follow(&mut self) {
        let wanted_groups: HashSet<DataGroup> = lock(&self.wanted)
            .values()
            .flat_map(|entry| entry.groups.iter().copied())
            .collect();
        let any_wanted = !wanted_groups.is_empty();
        match (any_wanted, self.hub.is_some()) {
            (true, false) => {
                let (wanted, sample, deliver) = (
                    Arc::clone(&self.wanted),
                    Arc::clone(&self.sample),
                    Arc::clone(&self.deliver),
                );
                self.hub = start_module("data hub", any_wanted, || {
                    DataHub::start(wanted, sample, deliver).map_err(|error| PlatformError::Failed {
                        context: "data hub",
                        message: error.to_string(),
                    })
                });
            }
            (false, true) => {
                // Dropping the hub joins its worker, so nothing samples while the groups are let go below.
                self.hub = None;
                module_off("data hub");
                self.release_groups(DataGroup::ALL);
                self.held.clear();
            }
            _ => {}
        }
        // A group whose last page went while the hub goes on (a page that needed only the date stays): the sampler lets
        // go of it now. A pass that began before this change can still read it once more; the release when the hub
        // stops catches that.
        let dropped: Vec<DataGroup> = self.held.difference(&wanted_groups).copied().collect();
        self.release_groups(dropped);
        self.held = wanted_groups;
    }

    fn release_groups(&self, groups: impl IntoIterator<Item = DataGroup>) {
        if let Some(release) = &self.release {
            for group in groups {
                release(group);
            }
        }
    }
}

/// How long a source that failed (WMI unavailable, a counter missing) is left alone before the next attempt.
const RETRY_AFTER: Duration = Duration::from_secs(60);

/// What the sampler holds for one group that needs more than a single call.
enum Held<T> {
    /// Nothing yet: the next reading creates it.
    Empty,
    Ready(T),
    /// The source failed at this time; the group reports its legacy default until the retry.
    Failed(Instant),
}

impl<T> Held<T> {
    /// Whether a reading may try the source now.
    fn may_try(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Ready(_) => false,
            Self::Failed(at) => at.elapsed() >= RETRY_AFTER,
        }
    }
}

/// What the system sampler keeps between readings: the processor counters (open while a page wants the Processor group),
/// the loopback capture of the Audio group (open while a page wants Audio), the BIOS and motherboard texts (read once from
/// WMI) and which groups already logged a failure.
struct SystemState {
    processor: Held<ProcessorSampler>,
    audio: Held<AudioCapture>,
    /// The current media session, open while a page wants Audio next to the capture; it fills the media keys.
    media: Held<MediaSession>,
    bios: Held<Value>,
    motherboard: Held<Value>,
    logged: HashSet<DataGroup>,
}

impl SystemState {
    fn new() -> Self {
        Self {
            processor: Held::Empty,
            audio: Held::Empty,
            media: Held::Empty,
            bios: Held::Empty,
            motherboard: Held::Empty,
            logged: HashSet::new(),
        }
    }

    /// Logs the first failure of `group` and stays quiet about the repeats; the group then reports its default.
    fn failed(&mut self, group: DataGroup, error: &SysInfoError) {
        if self.logged.insert(group) {
            tracing::warn!(?group, %error, "the data group cannot be read, a page gets its empty default");
        }
    }

    /// Reads `group`. `None` when this build has no provider for it; a provider that fails gives the group's legacy
    /// default (`State` false and empty values) and is logged once.
    fn read(&mut self, group: DataGroup) -> Option<Value> {
        match group {
            DataGroup::Date => self.plain(group, sucrose_sysinfo::date_snapshot(), None),
            DataGroup::Memory => self.plain(
                group,
                sucrose_sysinfo::memory_snapshot(),
                Some(sucrose_sysinfo::memory_unavailable()),
            ),
            DataGroup::Processor => self.processor(),
            DataGroup::Audio => self.audio(),
            DataGroup::Bios => {
                let mut held = std::mem::replace(&mut self.bios, Held::Empty);
                let value = self.once(
                    group,
                    &mut held,
                    sucrose_sysinfo::bios_snapshot,
                    sucrose_sysinfo::bios_unavailable,
                );
                self.bios = held;
                value
            }
            DataGroup::Motherboard => {
                let mut held = std::mem::replace(&mut self.motherboard, Held::Empty);
                let value = self.once(
                    group,
                    &mut held,
                    sucrose_sysinfo::motherboard_snapshot,
                    sucrose_sysinfo::motherboard_unavailable,
                );
                self.motherboard = held;
                value
            }
            _ => None,
        }
    }

    /// A group read by one call every time. `default` is what a page gets when the call fails; without one the group
    /// is skipped.
    fn plain(
        &mut self,
        group: DataGroup,
        snapshot: Result<Value, SysInfoError>,
        default: Option<Value>,
    ) -> Option<Value> {
        match snapshot {
            Ok(value) => Some(value),
            Err(SysInfoError::Unsupported(what)) => {
                tracing::debug!(?group, what, "the data group has no provider on this OS");
                None
            }
            Err(error) => {
                self.failed(group, &error);
                default
            }
        }
    }

    /// A group read from WMI once and kept: a success is kept for the life of the hub, a failure for [`RETRY_AFTER`].
    fn once(
        &mut self,
        group: DataGroup,
        held: &mut Held<Value>,
        read: fn() -> Result<Value, SysInfoError>,
        unavailable: fn() -> Value,
    ) -> Option<Value> {
        if held.may_try() {
            match read() {
                Ok(value) => *held = Held::Ready(value),
                Err(SysInfoError::Unsupported(what)) => {
                    tracing::debug!(?group, what, "the data group has no provider on this OS");
                    return None;
                }
                Err(error) => {
                    self.failed(group, &error);
                    *held = Held::Failed(Instant::now());
                }
            }
        }
        match held {
            Held::Ready(value) => Some(value.clone()),
            _ => Some(unavailable()),
        }
    }

    /// The Processor group: the counters are opened by the first reading and kept until the group is released.
    fn processor(&mut self) -> Option<Value> {
        if self.processor.may_try() {
            match ProcessorSampler::new() {
                Ok(mut sampler) => {
                    if let Some(error) = sampler.info_error() {
                        self.failed(DataGroup::Processor, &error);
                    }
                    self.processor = Held::Ready(sampler);
                }
                Err(SysInfoError::Unsupported(what)) => {
                    tracing::debug!(group = ?DataGroup::Processor, what, "the data group has no provider on this OS");
                    return None;
                }
                Err(error) => {
                    self.failed(DataGroup::Processor, &error);
                    self.processor = Held::Failed(Instant::now());
                }
            }
        }
        match &mut self.processor {
            Held::Ready(sampler) => Some(sampler.sample()),
            _ => Some(sucrose_sysinfo::processor_unavailable()),
        }
    }

    /// The Audio group: the loopback capture is opened by the first reading and kept until the group is released (D-24). A
    /// device that cannot be captured gives the legacy default and is tried again after [`RETRY_AFTER`].
    fn audio(&mut self) -> Option<Value> {
        if self.audio.may_try() {
            match AudioCapture::start() {
                Ok(capture) => {
                    tracing::info!(format = capture.format(), "audio capture started");
                    self.audio = Held::Ready(capture);
                }
                Err(SysInfoError::Unsupported(what)) => {
                    tracing::debug!(group = ?DataGroup::Audio, what, "the data group has no provider on this OS");
                    return None;
                }
                Err(error) => {
                    self.failed(DataGroup::Audio, &error);
                    self.audio = Held::Failed(Instant::now());
                }
            }
        }
        let media = self.media_fields();
        Some(match (&self.audio, media) {
            (Held::Ready(capture), Some(media)) => capture.snapshot_with(&media),
            (Held::Ready(capture), None) => capture.snapshot(),
            (_, Some(media)) => sucrose_sysinfo::audio_unavailable_with(&media),
            (_, None) => sucrose_sysinfo::audio_unavailable(),
        })
    }

    /// The media keys of the Audio group: the media session is opened by the first reading and kept until the group is
    /// released. `None` when there is no session manager (the group then keeps its legacy defaults for those keys).
    fn media_fields(&mut self) -> Option<serde_json::Map<String, Value>> {
        if self.media.may_try() {
            match MediaSession::start() {
                Ok(session) => {
                    tracing::info!("media session followed");
                    self.media = Held::Ready(session);
                }
                Err(SysInfoError::Unsupported(what)) => {
                    tracing::debug!(group = ?DataGroup::Audio, what, "no media session provider on this OS");
                    self.media = Held::Failed(Instant::now());
                }
                Err(error) => {
                    self.failed(DataGroup::Audio, &error);
                    self.media = Held::Failed(Instant::now());
                }
            }
        }
        match &self.media {
            Held::Ready(session) => Some(session.fields()),
            _ => None,
        }
    }

    /// No running page wants `group` any more: whatever is held for it goes (the counters close, a cached read is
    /// dropped, so a later page reads it afresh) and a failure may be logged again.
    fn release(&mut self, group: DataGroup) {
        match group {
            DataGroup::Processor => self.processor = Held::Empty,
            DataGroup::Audio => {
                if matches!(self.audio, Held::Ready(_)) {
                    tracing::info!("audio capture stopped");
                }
                // Dropping the capture stops its thread and lets the device go.
                self.audio = Held::Empty;
                // The media session goes with it: what is playing is followed only while a page wants Audio (T-04-64).
                if matches!(self.media, Held::Ready(_)) {
                    tracing::info!("media session released");
                }
                self.media = Held::Empty;
            }
            DataGroup::Bios => self.bios = Held::Empty,
            DataGroup::Motherboard => self.motherboard = Held::Empty,
            _ => {}
        }
        self.logged.remove(&group);
    }
}

/// The sampler of this build and the release that goes with it: the groups `sucrose-sysinfo` provides, as the JSON text
/// the legacy payload holds. Everything the sampler holds sits behind one lock, so a release from the service thread
/// waits for a reading in progress on the hub thread.
#[must_use]
pub fn system_sampling() -> (Sampler, Release) {
    let state = Arc::new(Mutex::new(SystemState::new()));
    let sample: Sampler = {
        let state = Arc::clone(&state);
        Arc::new(move |group| {
            let value = state.lock().unwrap_or_else(PoisonError::into_inner).read(group)?;
            serde_json::to_string(&value).ok()
        })
    };
    let release: Release = Arc::new(move |group| {
        state.lock().unwrap_or_else(PoisonError::into_inner).release(group);
    });
    (sample, release)
}

/// The sampler of [`system_sampling`] without its release.
#[cfg(test)]
#[must_use]
pub fn system_sampler() -> Sampler {
    system_sampling().0
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    type Log = Arc<Mutex<Vec<(u8, Vec<DataGroup>)>>>;

    /// A sampler that counts the reads of each group and answers `{"g":"<Debug name>"}`.
    fn counting_sampler() -> (Sampler, Arc<Mutex<Vec<DataGroup>>>) {
        let reads = Arc::new(Mutex::new(Vec::new()));
        let kept = Arc::clone(&reads);
        let sample: Sampler = Arc::new(move |group| {
            kept.lock().expect("reads").push(group);
            Some(format!("{{\"g\":\"{group:?}\"}}"))
        });
        (sample, reads)
    }

    /// A deliverer that records what each slot was offered and answers with `answer(slot)`.
    fn recording_deliver(answer: impl Fn(u8) -> Delivery + Send + Sync + 'static) -> (Deliver, Log) {
        let log: Log = Arc::default();
        let kept = Arc::clone(&log);
        let deliver: Deliver = Arc::new(move |slot, batch| {
            kept.lock()
                .expect("log")
                .push((slot, batch.iter().map(|(group, _)| *group).collect()));
            answer(slot)
        });
        (deliver, log)
    }

    #[test]
    fn a_pass_reads_a_group_once_however_many_slots_want_it_and_each_slot_gets_its_own_groups() {
        let (sample, reads) = counting_sampler();
        let (deliver, log) = recording_deliver(|_| Delivery::Sent);
        let slots = vec![
            (0, vec![DataGroup::Date]),
            (1, vec![DataGroup::Date, DataGroup::Bios]),
            (2, vec![DataGroup::Bios]),
        ];

        pass(0, &slots, &sample, &deliver, &mut HashSet::new());

        assert_eq!(
            *reads.lock().expect("reads"),
            [DataGroup::Bios, DataGroup::Date],
            "one collector per group, in payload order"
        );
        assert_eq!(
            *log.lock().expect("log"),
            [
                (0, vec![DataGroup::Date]),
                (1, vec![DataGroup::Bios, DataGroup::Date]),
                (2, vec![DataGroup::Bios])
            ]
        );
    }

    #[test]
    fn a_pass_reads_only_the_groups_somebody_wants() {
        let (sample, reads) = counting_sampler();
        let (deliver, _log) = recording_deliver(|_| Delivery::Sent);

        pass(0, &[(0, vec![DataGroup::Date])], &sample, &deliver, &mut HashSet::new());
        pass(1, &[], &sample, &deliver, &mut HashSet::new());

        assert_eq!(*reads.lock().expect("reads"), [DataGroup::Date]);
    }

    #[test]
    fn a_group_is_read_at_its_own_cadence() {
        let (sample, reads) = counting_sampler();
        let (deliver, log) = recording_deliver(|_| Delivery::Sent);
        let slots = vec![(0, vec![DataGroup::Date, DataGroup::Memory])];

        for tick in 0..11 {
            pass(tick, &slots, &sample, &deliver, &mut HashSet::new());
        }

        let reads = reads.lock().expect("reads");
        assert_eq!(reads.iter().filter(|group| **group == DataGroup::Date).count(), 11);
        assert_eq!(
            reads.iter().filter(|group| **group == DataGroup::Memory).count(),
            2,
            "ticks 0 and 10"
        );
        assert_eq!(log.lock().expect("log").len(), 11);
    }

    #[test]
    fn a_full_queue_drops_the_snapshot_and_the_next_tick_sends_a_fresh_one() {
        let (sample, reads) = counting_sampler();
        let full = Arc::new(AtomicUsize::new(1));
        let counter = Arc::clone(&full);
        let (deliver, log) = recording_deliver(move |_| {
            if counter.fetch_sub(1, Ordering::SeqCst) > 0 {
                Delivery::Dropped
            } else {
                Delivery::Sent
            }
        });
        let slots = vec![(0, vec![DataGroup::Date])];

        pass(0, &slots, &sample, &deliver, &mut HashSet::new());
        pass(1, &slots, &sample, &deliver, &mut HashSet::new());

        assert_eq!(
            reads.lock().expect("reads").len(),
            2,
            "the hub read again, it did not wait"
        );
        assert_eq!(log.lock().expect("log").len(), 2);
    }

    #[test]
    fn a_group_without_a_provider_sends_nothing() {
        let sample: Sampler = Arc::new(|_| None);
        let (deliver, log) = recording_deliver(|_| Delivery::Sent);

        pass(
            0,
            &[(0, vec![DataGroup::Audio])],
            &sample,
            &deliver,
            &mut HashSet::new(),
        );

        assert!(log.lock().expect("log").is_empty());
    }

    #[test]
    fn the_system_sampler_gives_the_date_and_nothing_for_groups_without_a_provider() {
        let sample = system_sampler();

        assert!(sample(DataGroup::Battery).is_none(), "the battery has no provider yet");
        #[cfg(windows)]
        {
            let text = sample(DataGroup::Date).expect("the date reads");
            let value: serde_json::Value = serde_json::from_str(&text).expect("JSON");
            assert_eq!(value["State"], true);
            assert!(!text.contains('\n'), "compact, as the legacy payload");
        }
    }

    fn module() -> (DataModule, Arc<Mutex<Vec<DataGroup>>>, Log) {
        let (sample, reads) = counting_sampler();
        let (deliver, log) = recording_deliver(|_| Delivery::Sent);
        (DataModule::new(sample, deliver), reads, log)
    }

    #[test]
    fn the_hub_does_not_exist_until_a_renderer_wants_data() {
        let (module, reads, _log) = module();

        assert!(!module.running());
        thread::sleep(TICK * 3);
        assert!(
            reads.lock().expect("reads").is_empty(),
            "nothing is sampled while nobody wants data"
        );
    }

    #[test]
    fn an_empty_wish_starts_nothing() {
        let (mut module, _reads, _log) = module();

        module.want(0, 10, &[]);

        assert!(!module.running());
    }

    #[test]
    fn the_hub_runs_while_a_renderer_wants_data_and_stops_with_the_last_one() {
        let (mut module, reads, log) = module();

        module.want(0, 10, &[DataGroup::Date, DataGroup::Date]);
        module.want(1, 11, &[DataGroup::Date]);
        assert!(module.running());
        thread::sleep(TICK * 4);

        module.release(0, Some(10));
        assert!(module.running(), "the second renderer still wants data");
        module.release(1, None);
        assert!(!module.running());
        let after_stop = reads.lock().expect("reads").len();
        assert!(after_stop >= 2, "the hub ticked while it ran: {after_stop}");
        thread::sleep(TICK * 3);
        assert_eq!(
            reads.lock().expect("reads").len(),
            after_stop,
            "no sampling after the last renderer ended"
        );
        let log = log.lock().expect("log");
        assert!(log.iter().any(|(slot, _)| *slot == 0) && log.iter().any(|(slot, _)| *slot == 1));
    }

    #[test]
    fn the_exit_of_a_replaced_renderer_leaves_its_replacement_wanted() {
        let (mut module, _reads, _log) = module();
        module.want(0, 20, &[DataGroup::Date]);

        module.release(0, Some(10));

        assert!(module.running(), "pid 10 is not the renderer that asked");
        module.release(0, Some(20));
        assert!(!module.running());
    }

    #[test]
    fn a_later_wish_of_a_slot_replaces_the_earlier_one() {
        let (mut module, _reads, _log) = module();

        module.want(0, 10, &[DataGroup::Date]);
        module.want(0, 11, &[DataGroup::Memory]);

        assert_eq!(lock(&module.wanted).get(&0).map(|entry| entry.pid), Some(11));
        assert_eq!(
            lock(&module.wanted).get(&0).map(|entry| entry.groups.clone()),
            Some(vec![DataGroup::Memory])
        );
    }

    #[test]
    fn the_date_is_asked_for_every_tick() {
        assert_eq!(every_ticks(DataGroup::Date), 1);
    }

    #[test]
    fn memory_processor_bios_and_motherboard_are_asked_for_once_a_second() {
        for group in [
            DataGroup::Memory,
            DataGroup::Processor,
            DataGroup::Bios,
            DataGroup::Motherboard,
        ] {
            assert_eq!(every_ticks(group), 10, "{group:?}");
        }
    }

    /// A release that records the groups it was called for.
    fn recording_release() -> (Release, Arc<Mutex<Vec<DataGroup>>>) {
        let released = Arc::new(Mutex::new(Vec::new()));
        let kept = Arc::clone(&released);
        let release: Release = Arc::new(move |group| kept.lock().expect("released").push(group));
        (release, released)
    }

    #[test]
    fn a_group_is_released_when_its_last_renderer_stops_wanting_it_while_the_hub_goes_on() {
        let (sample, _reads) = counting_sampler();
        let (deliver, _log) = recording_deliver(|_| Delivery::Sent);
        let (release, released) = recording_release();
        let mut module = DataModule::new(sample, deliver).with_release(release);

        module.want(0, 10, &[DataGroup::Date, DataGroup::Processor]);
        module.want(1, 11, &[DataGroup::Processor]);
        module.want(0, 10, &[DataGroup::Date]);
        assert!(
            released.lock().expect("released").is_empty(),
            "renderer 1 still wants the processor"
        );
        module.release(1, None);

        assert!(module.running(), "renderer 0 still wants the date");
        assert_eq!(*released.lock().expect("released"), [DataGroup::Processor]);
    }

    #[test]
    fn every_group_is_released_when_the_hub_stops() {
        let (sample, _reads) = counting_sampler();
        let (deliver, _log) = recording_deliver(|_| Delivery::Sent);
        let (release, released) = recording_release();
        let mut module = DataModule::new(sample, deliver).with_release(release);
        module.want(0, 10, &[DataGroup::Memory]);

        module.release(0, None);

        assert!(!module.running());
        let released = released.lock().expect("released");
        assert!(
            DataGroup::ALL.iter().all(|group| released.contains(group)),
            "{released:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn the_system_sampler_reads_bios_and_motherboard_once_and_keeps_them_until_released() {
        let (sample, release) = system_sampling();

        for group in [DataGroup::Bios, DataGroup::Motherboard] {
            let first: serde_json::Value =
                serde_json::from_str(&sample(group).expect("WMI group reads")).expect("JSON");
            assert_eq!(first["State"], true, "{group:?}");
            let again: serde_json::Value =
                serde_json::from_str(&sample(group).expect("kept group reads")).expect("JSON");
            assert_eq!(first, again, "{group:?} is read once and kept");
            release(group);
            assert!(sample(group).is_some(), "{group:?} is read afresh after the release");
        }
    }

    #[test]
    fn a_failed_one_time_read_gives_the_legacy_default_and_is_not_retried_at_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
        fn failing() -> Result<serde_json::Value, SysInfoError> {
            ATTEMPTS.fetch_add(1, Ordering::SeqCst);
            Err(SysInfoError::Os {
                call: "WMI query",
                message: "no WMI".to_owned(),
            })
        }
        let mut state = SystemState::new();
        let mut held = Held::Empty;

        let first = state.once(DataGroup::Bios, &mut held, failing, sucrose_sysinfo::bios_unavailable);
        let second = state.once(DataGroup::Bios, &mut held, failing, sucrose_sysinfo::bios_unavailable);

        assert_eq!(
            first.as_ref().map(|value| &value["State"]),
            Some(&serde_json::json!(false))
        );
        assert_eq!(first, second);
        assert_eq!(
            ATTEMPTS.load(Ordering::SeqCst),
            1,
            "the failure is kept for the retry delay"
        );
        assert!(state.logged.contains(&DataGroup::Bios), "the failure was logged once");
    }

    #[cfg(windows)]
    #[test]
    fn the_system_sampler_reads_memory_and_processor_and_releases_the_counters() {
        let (sample, release) = system_sampling();

        let memory: serde_json::Value =
            serde_json::from_str(&sample(DataGroup::Memory).expect("memory reads")).expect("JSON");
        assert_eq!(memory["State"], true);
        assert!(memory["MemoryLoad"].as_f64().is_some());
        let first: serde_json::Value =
            serde_json::from_str(&sample(DataGroup::Processor).expect("processor reads")).expect("JSON");
        assert!(first["Cores"].is_array());
        thread::sleep(Duration::from_millis(1100));
        let second: serde_json::Value =
            serde_json::from_str(&sample(DataGroup::Processor).expect("processor reads")).expect("JSON");
        assert_eq!(
            second["Type"], "Timer100NsInverse",
            "the second reading has real figures"
        );

        release(DataGroup::Processor);
        let again = sample(DataGroup::Processor).expect("a later page opens the counters afresh");
        assert!(again.contains("\"Cores\""));
    }

    #[test]
    fn audio_is_released_when_its_last_page_stops_wanting_it_and_the_other_groups_go_on() {
        let (sample, _reads) = counting_sampler();
        let (deliver, _log) = recording_deliver(|_| Delivery::Sent);
        let (release, released) = recording_release();
        let mut module = DataModule::new(sample, deliver).with_release(release);

        module.want(0, 10, &[DataGroup::Date, DataGroup::Audio]);
        module.want(1, 11, &[DataGroup::Audio]);
        module.want(0, 10, &[DataGroup::Date]);
        assert!(
            released.lock().expect("released").is_empty(),
            "page 1 still wants the audio"
        );
        module.release(1, None);

        assert!(module.running(), "page 0 still wants the date");
        assert_eq!(*released.lock().expect("released"), [DataGroup::Audio]);
    }

    #[test]
    fn a_page_that_wants_audio_again_gets_it_read_afresh_after_the_release() {
        let (sample, reads) = counting_sampler();
        let (deliver, _log) = recording_deliver(|_| Delivery::Sent);
        let (release, released) = recording_release();
        let mut module = DataModule::new(sample, deliver).with_release(release);

        module.want(0, 10, &[DataGroup::Audio]);
        module.release(0, None);
        assert!(
            !module.running(),
            "no page wants anything: the hub and the capture are gone"
        );
        assert!(released.lock().expect("released").contains(&DataGroup::Audio));
        module.want(0, 12, &[DataGroup::Audio]);

        assert!(module.running());
        thread::sleep(TICK * 3);
        assert!(
            reads.lock().expect("reads").contains(&DataGroup::Audio),
            "the new page's hub reads the audio group"
        );
    }

    #[test]
    fn releasing_audio_forgets_a_failed_start_so_the_next_page_tries_at_once() {
        let mut state = SystemState::new();
        state.audio = Held::Failed(Instant::now());
        assert!(!state.audio.may_try(), "a failure waits for the retry delay");

        state.release(DataGroup::Audio);

        assert!(state.audio.may_try());
    }

    #[cfg(windows)]
    #[test]
    fn the_system_sampler_gives_the_audio_group_and_lets_the_capture_go_on_release() {
        let (sample, release) = system_sampling();

        let first: serde_json::Value =
            serde_json::from_str(&sample(DataGroup::Audio).expect("the audio group reads")).expect("JSON");
        thread::sleep(Duration::from_millis(300));
        let second: serde_json::Value =
            serde_json::from_str(&sample(DataGroup::Audio).expect("the audio group reads")).expect("JSON");

        for value in [&first, &second] {
            // A machine with a render device gives 128 bands; one without gives the legacy default.
            let bands = value["Data"].as_array().map(Vec::len);
            assert!(bands == Some(128) || bands == Some(0), "{bands:?}");
            assert_eq!(value["DataLeft"].as_array().map(Vec::len), bands);
            assert_eq!(value["DataRight"].as_array().map(Vec::len), bands);
        }
        release(DataGroup::Audio);
        assert!(
            sample(DataGroup::Audio).is_some(),
            "a later page opens the capture afresh"
        );
        release(DataGroup::Audio);
    }
}
