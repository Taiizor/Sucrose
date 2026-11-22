//! Starts the renderer processes, reports when one ends and restarts it after a pause.
//!
//! Every renderer runs in a kill-on-close job object without a console, so none can outlive the service
//! (T-02-23). A waiter thread per renderer blocks until its process ends and tells the service, so nothing polls.

use std::{
    collections::HashSet,
    path::PathBuf,
    process::{Child, Command},
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    thread,
    time::{Duration, Instant},
};

use anyhow::Context as _;
use sucrose_platform::RendererJob;

/// How long a crashed renderer's slot stays empty before the new renderer starts.
pub const RESTART_DELAY: Duration = Duration::from_millis(500);

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
    /// The pids of the processes this supervisor started and that still run: the only ones whose Hello is accepted.
    pids: HashSet<u32>,
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
        let mut command = Command::new(&self.exe);
        command.arg("--render").arg(slot.to_string());
        let child = self.job.spawn(&mut command).context("cannot start the renderer")?;
        let pid = child.id();
        state.pids.insert(pid);
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

    /// The pids of the renderers that run now.
    pub fn pids(&self) -> HashSet<u32> {
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
                if this.lock().closing {
                    return;
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
