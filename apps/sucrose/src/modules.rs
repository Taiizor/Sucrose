//! Optional service modules (D-008, ARCH-03): a module the user turns off starts nothing, not a timer that returns
//! early. Every module logs one line when it is started or found off, with its name (tray, fullscreen, memory).

use std::collections::HashMap;

use sucrose_ipc::portal::MemoryReport;
use sucrose_platform::PlatformError;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::topology::TimerCommand;

/// The loop timer of the memory report (1 and 2 belong to the topology debounce, 3 to the fullscreen rule).
pub const MEMORY_TIMER: u32 = 4;

/// How often the Portal is told the memory of the Sucrose processes (D-19).
pub const MEMORY_PERIOD_MS: u32 = 1000;

/// Starts the module `name` when `enabled`. With `enabled` false `create` is never called: no thread, no timer, no
/// icon and none of their resources exist (D-008). A module that cannot be created is logged and the service runs
/// without it.
pub fn start_module<T>(
    name: &'static str,
    enabled: bool,
    create: impl FnOnce() -> Result<T, PlatformError>,
) -> Option<T> {
    if !enabled {
        module_off(name);
        return None;
    }
    match create() {
        Ok(module) => {
            tracing::info!(name, "module started");
            Some(module)
        }
        Err(error) => {
            tracing::error!(name, %error, "module failed, the service runs without it");
            None
        }
    }
}

/// Logs that the module `name` is off: it was never wanted, or it has just been stopped.
pub fn module_off(name: &'static str) {
    tracing::info!(name, "module off");
}

/// The renderer pids in slot order, so the Portal's list does not reorder from one report to the next.
pub fn pids_by_slot(running: &HashMap<u32, u8>) -> Vec<u32> {
    let mut by_slot: Vec<(u8, u32)> = running.iter().map(|(pid, slot)| (*slot, *pid)).collect();
    by_slot.sort_unstable();
    by_slot.into_iter().map(|(_, pid)| pid).collect()
}

/// Reads the memory of the Sucrose processes. It holds the OS process list only while it exists, so with no Portal
/// connected nothing of it is allocated.
pub struct MemorySampler {
    system: System,
}

impl MemorySampler {
    /// Opens the OS process information.
    pub fn new() -> Result<Self, PlatformError> {
        let system = System::new().map_err(|error| PlatformError::Failed {
            context: "process memory",
            message: error.to_string(),
        })?;
        Ok(Self { system })
    }

    /// The memory of the service (`own`), of each renderer and of the Portal, in bytes (the working set on Windows,
    /// the convention of docs/MEASUREMENTS.md). Only the given pids are read, so no other process is touched and a
    /// process of the C# app is never counted (Pitfall 10). A process that is gone reads 0.
    pub fn sample(&mut self, own: u32, renderers: &[u32], portal: Option<u32>) -> MemoryReport {
        let mut wanted: Vec<Pid> = std::iter::once(own)
            .chain(renderers.iter().copied())
            .chain(portal)
            .map(Pid::from_u32)
            .collect();
        wanted.sort_unstable();
        wanted.dedup();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&wanted),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let read = |pid: u32| {
            self.system
                .process(Pid::from_u32(pid))
                .map_or(0, sysinfo::Process::memory)
        };
        MemoryReport {
            service: read(own),
            renderers: renderers.iter().map(|pid| read(*pid)).collect(),
            portal: portal.map_or(0, read),
        }
    }
}

/// The memory report module: it exists only while a Portal is connected (D-19).
#[derive(Default)]
pub struct MemoryModule {
    sampler: Option<MemorySampler>,
    /// Whether a Portal was connected when `follow` last changed, so a start that failed is not retried (and logged)
    /// on every message of the loop but only for the next Portal.
    connected: bool,
}

impl MemoryModule {
    /// Starts or stops the module to match whether a Portal is connected; the timer to arm or kill, if any.
    pub fn follow(&mut self, portal_connected: bool) -> Option<TimerCommand> {
        if portal_connected == self.connected {
            return None;
        }
        self.connected = portal_connected;
        if portal_connected {
            self.sampler = start_module("memory", true, MemorySampler::new);
            self.sampler.is_some().then_some(TimerCommand::Arm {
                id: MEMORY_TIMER,
                ms: MEMORY_PERIOD_MS,
            })
        } else {
            self.sampler.take().map(|_| {
                module_off("memory");
                TimerCommand::Kill(MEMORY_TIMER)
            })
        }
    }

    /// One reading, or `None` when the module is off.
    pub fn sample(&mut self, own: u32, renderers: &[u32], portal: Option<u32>) -> Option<MemoryReport> {
        self.sampler
            .as_mut()
            .map(|sampler| sampler.sample(own, renderers, portal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_app_visible_false_no_tray_is_built() {
        let mut built = false;

        let tray = start_module("tray", false, || {
            built = true;
            Ok(())
        });

        assert!(tray.is_none());
        assert!(!built, "the tray must not be created at all");
    }

    #[test]
    fn with_app_visible_true_the_tray_is_built() {
        assert_eq!(start_module("tray", true, || Ok(7)), Some(7));
    }

    #[test]
    fn a_tray_that_cannot_be_created_leaves_the_service_without_one() {
        let tray: Option<()> = start_module("tray", true, || Err(PlatformError::Unsupported("tray icon")));

        assert!(tray.is_none());
    }

    #[test]
    fn renderer_pids_come_in_slot_order() {
        let running = HashMap::from([(900, 2), (100, 0), (500, 1)]);

        assert_eq!(pids_by_slot(&running), vec![100, 500, 900]);
    }

    #[test]
    fn the_sampler_reads_this_process_and_reports_a_missing_one_as_zero() {
        let mut sampler = MemorySampler::new().expect("the process information opens");

        let report = sampler.sample(std::process::id(), &[u32::MAX - 1], Some(u32::MAX - 2));

        assert!(report.service > 0, "the service figure is above 0: {report:?}");
        assert_eq!(report.renderers, vec![0]);
        assert_eq!(report.portal, 0);
    }

    #[test]
    fn the_memory_module_exists_only_while_a_portal_is_connected() {
        let mut module = MemoryModule::default();
        assert_eq!(module.follow(false), None, "no Portal: nothing starts");
        assert!(module.sample(std::process::id(), &[], None).is_none());

        assert_eq!(
            module.follow(true),
            Some(TimerCommand::Arm {
                id: MEMORY_TIMER,
                ms: MEMORY_PERIOD_MS
            })
        );
        assert_eq!(module.follow(true), None, "already running");
        assert!(module.sample(std::process::id(), &[], None).is_some());

        assert_eq!(module.follow(false), Some(TimerCommand::Kill(MEMORY_TIMER)));
        assert!(module.sample(std::process::id(), &[], None).is_none());
        assert_eq!(module.follow(false), None);
    }
}
