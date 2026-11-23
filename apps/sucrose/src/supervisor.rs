//! Starts the renderer processes, reports when one ends and restarts it after a pause.
//!
//! Every renderer runs in a kill-on-close job object without a console, so none can outlive the service
//! (T-02-23). A waiter thread per renderer blocks until its process ends and tells the service, so nothing polls.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    process::{Child, Command},
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    thread,
    time::{Duration, Instant},
};

use anyhow::Context as _;
use sucrose_platform::RendererJob;

/// Exit code of a renderer that ended cleanly after Close.
pub const CLEAN: u8 = 0;
/// Exit code of a renderer whose engine is missing or cannot be loaded (D-20).
pub const ENGINE_MISSING: u8 = 2;
/// Exit code of a renderer that lost its desktop surface (the shell restarted or the monitors changed).
pub const SURFACE_LOST: u8 = 3;
/// Exit code of a renderer that could not start its wallpaper.
pub const START_FAILED: u8 = 4;
/// Exit code of a renderer whose message loop failed. [`classify_exit`] counts it as a crash, so the budget applies.
pub const LOOP_FAILED: u8 = 5;

/// The window the restart budgets are counted in.
const WINDOW: Duration = Duration::from_secs(60);
/// Crashes one monitor may have within [`WINDOW`] and still be restarted (D-22).
const MAX_CRASHES: usize = 3;
/// Lost surfaces one monitor may have within [`WINDOW`] and still be restarted.
const MAX_SURFACE_LOSSES: usize = 5;
/// The pause before the first restart; it doubles up to [`MAX_BACKOFF`].
const FIRST_BACKOFF: Duration = Duration::from_millis(500);
const MAX_BACKOFF: Duration = Duration::from_secs(2);

/// Why a renderer ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitClass {
    /// Ended after Close: expected.
    Clean,
    /// The engine is missing or cannot be loaded: a restart would fail the same way.
    EngineMissing,
    /// The wallpaper could not start.
    StartFailed,
    /// The desktop surface was lost; the renderer reports it so the service starts a fresh one.
    SurfaceLost,
    /// Crashed or was killed, with the exit code (-1 when the OS reports none).
    Crashed(i32),
}

/// Maps the exit code of a renderer to the reason it ended. A killed process (code 1) and an NTSTATUS crash code
/// both count as crashes.
pub fn classify_exit(code: Option<i32>) -> ExitClass {
    match code {
        Some(code) if code == i32::from(CLEAN) => ExitClass::Clean,
        Some(code) if code == i32::from(ENGINE_MISSING) => ExitClass::EngineMissing,
        Some(code) if code == i32::from(SURFACE_LOST) => ExitClass::SurfaceLost,
        Some(code) if code == i32::from(START_FAILED) => ExitClass::StartFailed,
        Some(code) => ExitClass::Crashed(code),
        None => ExitClass::Crashed(-1),
    }
}

/// What to do about an ended renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Leave the slot empty: the exit was expected or a restart cannot help.
    Leave,
    /// Start it again after the pause.
    Restart(Duration),
    /// The budget is spent: leave the slot empty and say why.
    GiveUp(Exhausted),
}

/// Which budget ran out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exhausted {
    /// Too many crashes.
    Crashes,
    /// The surface was lost too often.
    SurfaceLost,
}

impl Exhausted {
    /// One English sentence for the log and the tooltip.
    pub fn reason(self) -> String {
        match self {
            Self::Crashes => format!(
                "the renderer stopped after {MAX_CRASHES} crashes in {} s",
                WINDOW.as_secs()
            ),
            Self::SurfaceLost => format!(
                "the desktop surface was lost {MAX_SURFACE_LOSSES} times in {} s",
                WINDOW.as_secs()
            ),
        }
    }
}

/// The events of one kind inside the sliding window, with the cap they may reach.
#[derive(Debug)]
struct Window {
    max: usize,
    events: VecDeque<Instant>,
}

impl Window {
    fn new(max: usize) -> Self {
        Self {
            max,
            events: VecDeque::new(),
        }
    }

    /// Records an event at `now` and returns the pause before the restart: 0.5 s, 1 s, then 2 s. `None` when the
    /// cap is reached; a refused event is not recorded.
    fn allow(&mut self, now: Instant) -> Option<Duration> {
        while let Some(oldest) = self.events.front() {
            if now.saturating_duration_since(*oldest) >= WINDOW {
                self.events.pop_front();
            } else {
                break;
            }
        }
        if self.events.len() >= self.max {
            return None;
        }
        self.events.push_back(now);
        let doublings = u32::try_from(self.events.len() - 1).unwrap_or(u32::MAX).min(2);
        Some((FIRST_BACKOFF * 2_u32.pow(doublings)).min(MAX_BACKOFF))
    }
}

/// How many restarts one monitor gets (D-22): 3 crashes per 60 s, and a separate cap of 5 lost surfaces per 60 s so a
/// broken shell cannot loop forever either.
#[derive(Debug)]
pub struct RestartBudget {
    crashes: Window,
    surface_losses: Window,
}

impl Default for RestartBudget {
    fn default() -> Self {
        Self::new()
    }
}

impl RestartBudget {
    /// A fresh budget.
    pub fn new() -> Self {
        Self {
            crashes: Window::new(MAX_CRASHES),
            surface_losses: Window::new(MAX_SURFACE_LOSSES),
        }
    }

    /// Records a crash at `now`; the pause before the restart, or `None` when the budget is spent.
    pub fn allow(&mut self, now: Instant) -> Option<Duration> {
        self.crashes.allow(now)
    }

    /// Records a lost surface at `now`; the pause before the restart, or `None` when the cap is reached.
    pub fn allow_surface_lost(&mut self, now: Instant) -> Option<Duration> {
        self.surface_losses.allow(now)
    }
}

/// Decides what to do about a renderer that ended with `class` at `now`. A clean exit and a missing engine never
/// restart; a failed start counts like a crash.
pub fn decide(budget: &mut RestartBudget, class: ExitClass, now: Instant) -> Decision {
    match class {
        ExitClass::Clean | ExitClass::EngineMissing => Decision::Leave,
        ExitClass::StartFailed | ExitClass::Crashed(_) => budget
            .allow(now)
            .map_or(Decision::GiveUp(Exhausted::Crashes), Decision::Restart),
        ExitClass::SurfaceLost => budget
            .allow_surface_lost(now)
            .map_or(Decision::GiveUp(Exhausted::SurfaceLost), Decision::Restart),
    }
}

/// How long after `TaskbarCreated` a lost surface is counted as part of the shell restart and not held against the
/// renderer's budget.
pub const SHELL_RESTART_WINDOW: Duration = Duration::from_secs(30);
/// Shell restarts the service acts on per minute: `TaskbarCreated` is a broadcast any process can send, so a flood
/// of them must not keep the renderers in an endless close and start loop (T-02-26).
const MAX_SHELL_RESTARTS: usize = 5;

/// The recovery after explorer.exe restarted: it took Progman and with it every renderer's surface, so recovery
/// means new renderers, not re-parenting. Triggered only by the `TaskbarCreated` message, never by polling (D-11).
#[derive(Debug)]
pub struct ShellRestart {
    /// The end of the window in which a lost surface is expected; `None` when no restart was seen yet.
    until: Option<Instant>,
    /// Slots that were told to close and are started again once their renderer has ended, each exactly once.
    respawn: HashSet<u8>,
    /// How many restarts were acted on lately.
    recent: Window,
}

impl Default for ShellRestart {
    fn default() -> Self {
        Self {
            until: None,
            respawn: HashSet::new(),
            recent: Window::new(MAX_SHELL_RESTARTS),
        }
    }
}

impl ShellRestart {
    /// The shell restarted at `now`. `closing` are the slots whose renderer is being told to close and has to be
    /// started again. `false` when restarts come too fast and this one is ignored.
    pub fn begin(&mut self, now: Instant, closing: impl IntoIterator<Item = u8>) -> bool {
        if self.recent.allow(now).is_none() {
            return false;
        }
        self.until = Some(now + SHELL_RESTART_WINDOW);
        self.respawn = closing.into_iter().collect();
        true
    }

    /// Whether a shell restart began lately enough that a lost surface is part of it.
    pub fn pending(&self, now: Instant) -> bool {
        self.until.is_some_and(|until| now < until)
    }

    /// The renderer of `slot` is gone for another reason: it is not started again by the restart.
    pub fn forget(&mut self, slot: u8) {
        self.respawn.remove(&slot);
    }
}

/// [`decide`] with the shell restart taken into account: a renderer that was closed for the restart is started
/// again whatever its exit code, and a lost surface inside the restart window is not a crash and not a lost-surface
/// strike. Both start the new renderer after the shortest pause and leave the budgets untouched.
pub fn decide_exit(
    budget: &mut RestartBudget,
    shell: &mut ShellRestart,
    class: ExitClass,
    slot: u8,
    now: Instant,
) -> Decision {
    let expected = shell.respawn.remove(&slot);
    if class == ExitClass::EngineMissing {
        return Decision::Leave;
    }
    if expected || (class == ExitClass::SurfaceLost && shell.pending(now)) {
        return Decision::Restart(FIRST_BACKOFF);
    }
    decide(budget, class, now)
}

/// A renderer process ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exited {
    /// The slot the renderer served.
    pub slot: u8,
    /// The exit code; `None` when the OS reports none.
    pub code: Option<i32>,
}

/// Where the supervisor reports ended renderers. It runs on the waiter thread and must only queue the event.
type ExitSink = Arc<dyn Fn(Exited) + Send + Sync>;

#[derive(Default)]
struct State {
    /// The processes this supervisor started and that still run, with the slot each was started for: the only ones
    /// whose Hello is accepted, and only for that slot.
    pids: HashMap<u32, u8>,
    /// The slots that have a running renderer.
    running: HashSet<u8>,
    /// Set when shutdown begins: nothing is started any more.
    closing: bool,
}

/// Owns the job object and the renderer processes of one service.
pub struct Supervisor {
    job: RendererJob,
    exe: PathBuf,
    sink: ExitSink,
    state: Mutex<State>,
    /// Signalled when a renderer ends.
    ended: Condvar,
}

impl Supervisor {
    /// Creates the job object. `sink` receives every renderer that ends, from a waiter thread.
    pub fn new(sink: impl Fn(Exited) + Send + Sync + 'static) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            job: RendererJob::new().context("cannot create the renderer job object")?,
            exe: std::env::current_exe().context("cannot find the path of this executable")?,
            sink: Arc::new(sink),
            state: Mutex::default(),
            ended: Condvar::new(),
        }))
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Starts `sucrose --render <slot>` in the job and watches it.
    pub fn spawn(self: &Arc<Self>, slot: u8) -> anyhow::Result<()> {
        // Held across the spawn so a fast renderer cannot say Hello before its pid is recorded.
        let mut state = self.lock();
        if state.closing {
            anyhow::bail!("the service is shutting down");
        }
        // One renderer per slot, so a late restart can never double a monitor's renderer.
        if state.running.contains(&slot) {
            anyhow::bail!("slot {slot} already has a running renderer");
        }
        let mut command = Command::new(&self.exe);
        command.arg("--render").arg(slot.to_string());
        let child = self.job.spawn(&mut command).context("cannot start the renderer")?;
        let pid = child.id();
        state.pids.insert(pid, slot);
        state.running.insert(slot);
        tracing::info!(pid, slot, "renderer spawned");

        // The thread takes the child out of this cell; if it cannot start, the child is taken back and stopped.
        let cell = Arc::new(Mutex::new(Some(child)));
        let waiter_cell = Arc::clone(&cell);
        let this = Arc::clone(self);
        let started = thread::Builder::new()
            .name(format!("renderer-wait-{slot}"))
            .spawn(move || {
                let child = waiter_cell.lock().unwrap_or_else(PoisonError::into_inner).take();
                if let Some(child) = child {
                    this.wait_for(child, slot);
                }
            });
        if let Err(error) = started {
            state.pids.remove(&pid);
            state.running.remove(&slot);
            if let Some(mut child) = cell.lock().unwrap_or_else(PoisonError::into_inner).take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            return Err(error).context("cannot start the thread that waits for the renderer");
        }
        Ok(())
    }

    /// Blocks until the renderer ends, then reports it.
    fn wait_for(&self, mut child: Child, slot: u8) {
        let pid = child.id();
        let code = match child.wait() {
            Ok(status) => {
                tracing::info!(pid, slot, %status, "a renderer exited");
                status.code()
            }
            Err(error) => {
                tracing::warn!(%error, pid, slot, "cannot wait for a renderer");
                None
            }
        };
        {
            let mut state = self.lock();
            state.pids.remove(&pid);
            state.running.remove(&slot);
        }
        self.ended.notify_all();
        (self.sink)(Exited { slot, code });
    }

    /// The pids of the renderers that run now, each with the slot it was started for.
    pub fn pids(&self) -> HashMap<u32, u8> {
        self.lock().pids.clone()
    }

    /// The slots that have a running renderer, ascending.
    pub fn slots(&self) -> Vec<u8> {
        let mut slots: Vec<u8> = self.lock().running.iter().copied().collect();
        slots.sort_unstable();
        slots
    }

    /// Stops all starting from now on: a restart that is still waiting does nothing.
    pub fn begin_shutdown(&self) {
        self.lock().closing = true;
    }

    /// Waits up to `timeout` for the renderer of `slot` to be gone; `true` if it is.
    pub fn wait_exit(&self, slot: u8, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self.lock();
        while state.running.contains(&slot) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            state = self
                .ended
                .wait_timeout(state, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        true
    }

    /// Ends every renderer that is still running. Used after the grace period, when only the ones that ignored Close
    /// are left.
    pub fn kill_survivors(&self) {
        if let Err(error) = self.job.terminate_all() {
            tracing::warn!(%error, "cannot stop the renderers");
        }
    }

    /// Starts the renderer of `slot` again after `delay`, unless shutdown began meanwhile. A renderer that cannot be
    /// started is reported like one that ended at once with `failed_code`.
    pub fn restart_after(self: &Arc<Self>, slot: u8, delay: Duration, failed_code: i32) {
        let this = Arc::clone(self);
        let started = thread::Builder::new()
            .name(format!("renderer-restart-{slot}"))
            .spawn(move || {
                thread::sleep(delay);
                {
                    let state = this.lock();
                    if state.closing {
                        return;
                    }
                    if state.running.contains(&slot) {
                        tracing::info!(slot, "the slot already has a renderer, no restart needed");
                        return;
                    }
                }
                match this.spawn(slot) {
                    Ok(()) => tracing::info!(slot, "renderer restarted"),
                    Err(error) => {
                        tracing::error!(%error, slot, "cannot restart the renderer");
                        (this.sink)(Exited {
                            slot,
                            code: Some(failed_code),
                        });
                    }
                }
            });
        if let Err(error) = started {
            tracing::error!(%error, slot, "cannot start the thread that restarts the renderer");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(base: Instant, s: u64) -> Instant {
        base + Duration::from_secs(s)
    }

    #[test]
    fn a_failed_message_loop_is_a_crash_and_is_restarted_within_the_budget() {
        assert_eq!(
            classify_exit(Some(i32::from(LOOP_FAILED))),
            ExitClass::Crashed(i32::from(LOOP_FAILED))
        );
        let mut budget = RestartBudget::new();
        let decision = decide(&mut budget, classify_exit(Some(i32::from(LOOP_FAILED))), Instant::now());
        assert!(matches!(decision, Decision::Restart(_)), "{decision:?}");
    }

    #[test]
    fn exit_codes_are_classified() {
        assert_eq!(classify_exit(Some(0)), ExitClass::Clean);
        assert_eq!(classify_exit(Some(2)), ExitClass::EngineMissing);
        assert_eq!(classify_exit(Some(4)), ExitClass::StartFailed);
        assert_eq!(classify_exit(Some(3)), ExitClass::SurfaceLost);
        assert_eq!(classify_exit(Some(1)), ExitClass::Crashed(1));
        assert_eq!(classify_exit(Some(-1_073_741_819)), ExitClass::Crashed(-1_073_741_819));
        assert!(matches!(classify_exit(None), ExitClass::Crashed(_)));
    }

    #[test]
    fn three_crashes_in_a_minute_are_restarted_with_growing_pauses_and_the_fourth_is_refused() {
        let t0 = Instant::now();
        let mut budget = RestartBudget::new();

        assert_eq!(budget.allow(t0), Some(Duration::from_millis(500)));
        assert_eq!(budget.allow(secs(t0, 10)), Some(Duration::from_secs(1)));
        assert_eq!(budget.allow(secs(t0, 20)), Some(Duration::from_secs(2)));
        assert_eq!(budget.allow(secs(t0, 30)), None);
    }

    #[test]
    fn the_window_slides() {
        let t0 = Instant::now();
        let mut budget = RestartBudget::new();
        for s in [0, 10, 20] {
            assert!(budget.allow(secs(t0, s)).is_some());
        }

        assert!(budget.allow(secs(t0, 59)).is_none());
        assert!(budget.allow(secs(t0, 61)).is_some(), "the first crash left the window");
    }

    #[test]
    fn a_lost_surface_does_not_use_the_crash_budget_and_has_its_own_cap_of_five() {
        let t0 = Instant::now();
        let mut budget = RestartBudget::new();

        for s in 0..5 {
            assert!(budget.allow_surface_lost(secs(t0, s)).is_some(), "loss {s}");
        }
        assert!(budget.allow_surface_lost(secs(t0, 5)).is_none());
        assert_eq!(budget.allow(secs(t0, 6)), Some(Duration::from_millis(500)));
    }

    #[test]
    fn a_clean_exit_and_a_missing_engine_are_never_restarted() {
        let t0 = Instant::now();
        let mut budget = RestartBudget::new();

        assert_eq!(decide(&mut budget, ExitClass::Clean, t0), Decision::Leave);
        assert_eq!(decide(&mut budget, ExitClass::EngineMissing, t0), Decision::Leave);
        assert_eq!(
            decide(&mut budget, ExitClass::Crashed(1), t0),
            Decision::Restart(Duration::from_millis(500)),
            "leaving did not use the budget"
        );
    }

    #[test]
    fn a_failed_start_counts_like_a_crash_and_the_budget_ends_in_a_give_up() {
        let t0 = Instant::now();
        let mut budget = RestartBudget::new();

        assert!(matches!(
            decide(&mut budget, ExitClass::StartFailed, t0),
            Decision::Restart(_)
        ));
        assert!(matches!(
            decide(&mut budget, ExitClass::Crashed(1), secs(t0, 1)),
            Decision::Restart(_)
        ));
        assert!(matches!(
            decide(&mut budget, ExitClass::StartFailed, secs(t0, 2)),
            Decision::Restart(_)
        ));
        assert_eq!(
            decide(&mut budget, ExitClass::Crashed(-1_073_741_819), secs(t0, 3)),
            Decision::GiveUp(Exhausted::Crashes)
        );
    }

    #[test]
    fn a_lost_surface_that_keeps_coming_back_ends_in_a_give_up() {
        let t0 = Instant::now();
        let mut budget = RestartBudget::new();

        for s in 0..5 {
            assert!(matches!(
                decide(&mut budget, ExitClass::SurfaceLost, secs(t0, s)),
                Decision::Restart(_)
            ));
        }
        assert_eq!(
            decide(&mut budget, ExitClass::SurfaceLost, secs(t0, 5)),
            Decision::GiveUp(Exhausted::SurfaceLost)
        );
    }

    #[test]
    fn the_give_up_reasons_name_the_limits() {
        assert_eq!(
            Exhausted::Crashes.reason(),
            "the renderer stopped after 3 crashes in 60 s"
        );
        assert_eq!(
            Exhausted::SurfaceLost.reason(),
            "the desktop surface was lost 5 times in 60 s"
        );
    }

    fn crashes_left(budget: &mut RestartBudget, at: Instant) -> usize {
        (0..10).take_while(|n| budget.allow(secs(at, *n)).is_some()).count()
    }

    #[test]
    fn shell_restart_respawns_each_closed_slot_once_whatever_its_exit_code() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();
        let mut budget = RestartBudget::new();
        assert!(shell.begin(t0, [0, 1]));

        let clean = decide_exit(&mut budget, &mut shell, ExitClass::Clean, 0, secs(t0, 1));
        let lost = decide_exit(&mut budget, &mut shell, ExitClass::SurfaceLost, 1, secs(t0, 1));
        let again = decide_exit(&mut budget, &mut shell, ExitClass::Clean, 0, secs(t0, 2));

        assert_eq!(clean, Decision::Restart(Duration::from_millis(500)));
        assert_eq!(lost, Decision::Restart(Duration::from_millis(500)));
        assert_eq!(again, Decision::Leave, "no second respawn for the same slot");
    }

    #[test]
    fn shell_restart_surface_lost_inside_the_window_does_not_touch_any_budget() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();
        let mut budget = RestartBudget::new();
        assert!(shell.begin(t0, []));

        for s in 0..20 {
            let decision = decide_exit(&mut budget, &mut shell, ExitClass::SurfaceLost, 0, secs(t0, s % 20));
            assert_eq!(decision, Decision::Restart(Duration::from_millis(500)), "loss {s}");
        }

        assert_eq!(crashes_left(&mut budget, t0), 3, "the crash budget is whole");
        assert!(
            budget.allow_surface_lost(secs(t0, 21)).is_some(),
            "so is the lost-surface cap"
        );
    }

    #[test]
    fn shell_restart_surface_lost_outside_the_window_uses_the_cap_of_five() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();
        let mut budget = RestartBudget::new();
        let late = secs(t0, 100);
        assert!(shell.begin(t0, []));
        assert!(!shell.pending(late));

        let decisions: Vec<Decision> = (0..6)
            .map(|n| decide_exit(&mut budget, &mut shell, ExitClass::SurfaceLost, 0, secs(late, n)))
            .collect();

        assert!(decisions[..5].iter().all(|d| matches!(d, Decision::Restart(_))));
        assert_eq!(decisions[5], Decision::GiveUp(Exhausted::SurfaceLost));
    }

    #[test]
    fn shell_restart_window_lasts_30_seconds() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();

        assert!(!shell.pending(t0), "no restart seen yet");
        assert!(shell.begin(t0, []));

        assert!(shell.pending(secs(t0, 29)));
        assert!(!shell.pending(secs(t0, 30)));
    }

    #[test]
    fn shell_restart_never_restarts_a_missing_engine() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();
        let mut budget = RestartBudget::new();
        assert!(shell.begin(t0, [0]));

        let decision = decide_exit(&mut budget, &mut shell, ExitClass::EngineMissing, 0, secs(t0, 1));

        assert_eq!(decision, Decision::Leave);
    }

    #[test]
    fn shell_restart_a_crash_inside_the_window_still_uses_the_crash_budget() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();
        let mut budget = RestartBudget::new();
        assert!(shell.begin(t0, []));

        let decision = decide_exit(&mut budget, &mut shell, ExitClass::Crashed(-1), 0, secs(t0, 1));

        assert_eq!(decision, Decision::Restart(Duration::from_millis(500)));
        assert_eq!(crashes_left(&mut budget, secs(t0, 1)), 2);
    }

    #[test]
    fn shell_restart_a_forgotten_slot_is_not_respawned() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();
        let mut budget = RestartBudget::new();
        assert!(shell.begin(t0, [0]));

        shell.forget(0);

        assert_eq!(
            decide_exit(&mut budget, &mut shell, ExitClass::Clean, 0, secs(t0, 60)),
            Decision::Leave
        );
    }

    #[test]
    fn shell_restart_a_flood_of_broadcasts_is_limited_to_five_a_minute() {
        let t0 = Instant::now();
        let mut shell = ShellRestart::default();

        let accepted = (0..20).filter(|n| shell.begin(secs(t0, *n), [0])).count();

        assert_eq!(accepted, 5);
        assert!(shell.begin(secs(t0, 61), [0]), "the window slides");
    }
}
