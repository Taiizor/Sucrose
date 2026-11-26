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
use sucrose_core::compatible::DataGroup;
use sucrose_platform::PlatformError;

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

/// How many ticks pass between two readings of `group`: the legacy rates (Date every tick, the performance groups once
/// a second, the battery every two seconds). Bios and Motherboard are read once and cached by their providers, so they
/// are asked for once a minute at most.
#[must_use]
pub fn every_ticks(group: DataGroup) -> u64 {
    match group {
        DataGroup::Date | DataGroup::Audio => 1,
        DataGroup::Memory | DataGroup::Processor | DataGroup::Graphic | DataGroup::Network | DataGroup::Storage => 10,
        DataGroup::Battery => 20,
        DataGroup::Bios | DataGroup::Motherboard => 600,
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
        }
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
        let any_wanted = !lock(&self.wanted).is_empty();
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
                self.hub = None;
                module_off("data hub");
            }
            _ => {}
        }
    }
}

/// The sampler of this build: the groups `sucrose-sysinfo` provides, as the JSON text the legacy payload holds. The
/// other groups follow with plan 04-11.
#[must_use]
pub fn system_sampler() -> Sampler {
    Arc::new(|group| {
        let snapshot = match group {
            DataGroup::Date => sucrose_sysinfo::date_snapshot(),
            _ => return None,
        };
        match snapshot {
            Ok(value) => serde_json::to_string(&value).ok(),
            Err(error) => {
                tracing::debug!(?group, %error, "the data group cannot be read");
                None
            }
        }
    })
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

        assert!(sample(DataGroup::Audio).is_none());
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
}
