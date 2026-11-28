//! The built-in GIF renderer behind the [`Engine`] trait. The decoder runs on a thread of its own and hands frames over a
//! bounded channel of 2, so one frame is in flight and the animation is never in memory as a whole. The presenter and the
//! frame timer belong to the thread that owns the desktop surface, so the engine is split in two, like CEF and
//! WebView2: [`GifEngine`] is the thread-safe handle the IPC threads call (it queues commands and wakes the loop), and
//! [`GifPump`] owns the presenter and the decoder and is the only thing that touches them.

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crossbeam_channel::{Receiver, Sender, TryRecvError, TrySendError, bounded};
use sucrose_core::settings::Stretch;
use sucrose_engine::{
    PauseLevel,
    engine::{Engine, EngineClock, EngineError, Started, SurfaceHooks, SurfacePump},
};
use sucrose_ipc::protocol::{Assignment, PlaybackSettings};
use sucrose_platform::LoopTimers;

use crate::{
    decode::{Animation, DEFAULT_DELAY_MS, DecodedFrame, Scaling, Source},
    fit,
    present::{PresentError, Presenter},
};

/// The engine's name in the log.
const NAME: &str = "BuiltInGif";

/// The id of the frame timer on the renderer's loop (CEF uses 1).
pub const GIF_TIMER: u32 = 2;

/// How many commands may wait for the surface thread; a full queue drops the newest and logs it, so the IPC thread
/// never blocks.
const QUEUE_CAPACITY: usize = 64;

/// How many decoded frames may wait for the presenter: with the one being decoded, two are in flight beyond the one
/// on screen (T-04-66).
const FRAME_CHANNEL: usize = 2;

/// How long `start` waits for the first frame before it gives up on the file.
#[cfg(windows)]
const FIRST_FRAME_WAIT: Duration = Duration::from_secs(8);

/// What a `gif fit` line was about: the stretch mode, the frame size and the surface size.
type FitKey = (Stretch, (u32, u32), (u32, u32));

/// What the decode thread sends the pump.
#[derive(Debug)]
enum Message {
    /// The next frame of the animation.
    Frame(DecodedFrame),
    /// The only frame of the file: shown once, with no frame timer. The decoder stopped after sending it.
    Still(DecodedFrame),
    /// The animation ended. `continues` says the decoder saw loop on and carries on with frame 0 by itself; otherwise
    /// it stopped.
    PassEnd { continues: bool },
    /// The decoder gave up, with the reason.
    Failed(String),
}

/// A command the IPC threads queue for the surface thread.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Pause {
        paused: bool,
        level: PauseLevel,
    },
    Loop(bool),
    Load(PathBuf),
    Stretch(Stretch),
    /// The group's time base: Unix milliseconds that the content position counts from (D-043).
    TimeBase(u64),
}

/// The frame that is on screen `elapsed_ms` after the time base and how long it still stays (milliseconds), for an
/// animation whose frames last `delays_ms` (a delay of 0 plays at the legacy 100 ms). With `looping` the animation
/// repeats and the position is the elapsed time within one pass; without it the last frame stays once the pass is
/// over, with nothing left of its delay. An empty list gives frame 0. Pure and bounded by the frame list: no division by
/// zero, whatever the delays are.
#[must_use]
pub fn frame_at(delays_ms: &[u32], elapsed_ms: u64, looping: bool) -> (usize, u32) {
    let delay = |ms: u32| u64::from(if ms == 0 { DEFAULT_DELAY_MS } else { ms });
    let Some(last) = delays_ms.len().checked_sub(1) else {
        return (0, 0);
    };
    let total: u64 = delays_ms.iter().map(|ms| delay(*ms)).sum();
    let position = if looping {
        elapsed_ms % total.max(1)
    } else if elapsed_ms >= total {
        return (last, 0);
    } else {
        elapsed_ms
    };
    let mut start = 0_u64;
    for (index, ms) in delays_ms.iter().enumerate() {
        let end = start + delay(*ms);
        if position < end {
            return (index, u32::try_from(end - position).unwrap_or(u32::MAX));
        }
        start = end;
    }
    (last, 0)
}

/// The current time as Unix milliseconds.
fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX))
}

/// The values the handle and the decode thread read from any thread.
#[derive(Debug)]
struct Shared {
    looping: AtomicBool,
    paused: AtomicBool,
}

/// What lets the pump start and stop its frame timer; the loop's timers on the renderer, a recorder in tests.
pub trait TimerPort {
    /// Starts (or restarts) the timer `id`.
    fn set(&self, id: u32, ms: u32);
    /// Stops the timer `id`.
    fn kill(&self, id: u32);
}

impl TimerPort for LoopTimers {
    fn set(&self, id: u32, ms: u32) {
        LoopTimers::set(self, id, ms);
    }

    fn kill(&self, id: u32) {
        LoopTimers::kill(self, id);
    }
}

type Wake = Arc<dyn Fn() + Send + Sync>;

/// One decode thread and the channel it fills. Dropping it stops the thread: the receiver goes first, so a thread
/// blocked on a full channel wakes with an error.
struct Decoder {
    frames: Option<Receiver<Message>>,
    thread: Option<JoinHandle<()>>,
}

impl Decoder {
    /// Starts decoding `source` for a surface of `bound`, skipping `skip` frames first, so the first frame sent is
    /// frame `skip`.
    fn spawn(
        source: Source,
        bound: (u32, u32),
        scaling: Scaling,
        skip: usize,
        looping: Arc<Shared>,
        wake: Wake,
    ) -> Self {
        let (sender, receiver) = bounded(FRAME_CHANNEL);
        let thread = thread::Builder::new()
            .name("gif-decode".to_owned())
            .spawn(move || decode_loop(&source, bound, scaling, skip, &looping, &sender, &wake))
            .map_err(|error| tracing::error!(%error, "cannot start the GIF decode thread"))
            .ok();
        Self {
            frames: Some(receiver),
            thread,
        }
    }

    fn receiver(&self) -> Option<&Receiver<Message>> {
        self.frames.as_ref()
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        self.frames = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::warn!("the GIF decode thread panicked");
        }
    }
}

/// The body of the decode thread. Every failure is sent as a [`Message::Failed`]; nothing here panics on a bad file.
fn decode_loop(
    source: &Source,
    bound: (u32, u32),
    scaling: Scaling,
    skip: usize,
    shared: &Shared,
    sender: &Sender<Message>,
    wake: &Wake,
) {
    let send = |message: Message| {
        let sent = sender.send(message).is_ok();
        if sent {
            wake();
        }
        sent
    };
    let mut animation = match Animation::open_scaled(source.clone(), bound, scaling) {
        Ok(animation) => animation,
        Err(error) => {
            send(Message::Failed(error.to_string()));
            return;
        }
    };
    if skip == 0 {
        let (width, height) = animation.canvas();
        tracing::info!(
            "gif opened format={:?} width={width} height={height} single={}",
            animation.format(),
            animation.is_single()
        );
    }
    match animation.skip(skip) {
        Ok(skipped) if skipped < skip => {
            // The animation is shorter than the frame index asked for (the file changed): the pass is over.
            let continues = shared.looping.load(Ordering::SeqCst);
            if !send(Message::PassEnd { continues }) || !continues {
                return;
            }
            if let Err(error) = animation.rewind() {
                send(Message::Failed(error.to_string()));
                return;
            }
        }
        Ok(_) => {}
        Err(error) => {
            send(Message::Failed(error.to_string()));
            return;
        }
    }
    loop {
        match animation.next_frame() {
            Ok(Some(frame)) if animation.is_single() => {
                // A still picture or a one-frame animation is sent once: nothing loops and no timer runs.
                send(Message::Still(frame));
                return;
            }
            Ok(Some(frame)) => {
                if !send(Message::Frame(frame)) {
                    return;
                }
            }
            Ok(None) => {
                let continues = shared.looping.load(Ordering::SeqCst);
                if !send(Message::PassEnd { continues }) || !continues {
                    return;
                }
                if let Err(error) = animation.rewind() {
                    send(Message::Failed(error.to_string()));
                    return;
                }
            }
            Err(error) => {
                send(Message::Failed(error.to_string()));
                return;
            }
        }
    }
}

/// The thread-safe handle of a built-in GIF renderer.
pub struct GifEngine {
    commands: Sender<Command>,
    wake: Wake,
    shared: Arc<Shared>,
}

impl std::fmt::Debug for GifEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GifEngine")
            .field("shared", &self.shared)
            .finish_non_exhaustive()
    }
}

impl GifEngine {
    /// Queues `command` for the surface thread and wakes it. A full queue drops it and says so.
    fn queue(&self, command: Command) {
        match self.commands.try_send(command) {
            Ok(()) => (self.wake)(),
            Err(TrySendError::Full(command)) => {
                tracing::warn!(?command, "the GIF command queue is full, the command is dropped");
            }
            Err(TrySendError::Disconnected(command)) => tracing::debug!(?command, "the GIF pump is gone"),
        }
    }

    /// Whether the last pause command paused.
    #[must_use]
    pub fn paused(&self) -> bool {
        self.shared.paused.load(Ordering::SeqCst)
    }

    /// Whether the animation loops, as last set.
    #[must_use]
    pub fn looping(&self) -> bool {
        self.shared.looping.load(Ordering::SeqCst)
    }
}

impl Engine for GifEngine {
    fn name(&self) -> &'static str {
        NAME
    }

    /// Light and Heavy are alike until Round D: the frame timer stops and the decode thread waits on its full channel,
    /// so a paused animation uses no CPU and keeps its frame.
    fn set_paused(&self, paused: bool, level: PauseLevel) -> Result<(), EngineError> {
        self.shared.paused.store(paused, Ordering::SeqCst);
        self.queue(Command::Pause { paused, level });
        Ok(())
    }

    /// A GIF has no sound.
    fn set_volume(&self, _volume: u8) -> Result<(), EngineError> {
        Ok(())
    }

    fn set_loop(&self, looping: bool) -> Result<(), EngineError> {
        self.shared.looping.store(looping, Ordering::SeqCst);
        self.queue(Command::Loop(looping));
        Ok(())
    }

    /// The four modes place the frame as the legacy WPF `Image` did (`fit::fit`); the surface thread applies it.
    fn set_stretch(&self, stretch: Stretch) -> Result<(), EngineError> {
        self.queue(Command::Stretch(stretch));
        Ok(())
    }

    fn load_source(&self, source: &Path, settings: &PlaybackSettings) -> Result<(), EngineError> {
        self.shared.looping.store(settings.looping, Ordering::SeqCst);
        self.queue(Command::Load(source.to_owned()));
        Ok(())
    }

    /// The pump moves to the frame the time base gives at every loop wrap (D-043).
    fn set_time_base(&self, epoch_unix_ms: u64) {
        self.queue(Command::TimeBase(epoch_unix_ms));
    }

    /// A GIF has no timeline the follower could use before Round C.
    fn clock(&self) -> Option<&dyn EngineClock> {
        None
    }
}

/// The surface-thread half of the renderer.
pub struct GifPump {
    presenter: Box<dyn Presenter>,
    timers: Box<dyn TimerPort>,
    commands: Receiver<Command>,
    shared: Arc<Shared>,
    wake: Wake,
    on_playing: Option<Arc<dyn Fn() + Send + Sync>>,
    source: Source,
    bound: (u32, u32),
    /// The stretch mode in force: the rectangles come from it and the decoder prepares frames for it.
    stretch: Stretch,
    /// The size of the frame on screen, for the `gif fit` log line.
    frame_size: Option<(u32, u32)>,
    /// What the `gif fit` line last said, so it is logged when the mode or a size changes and not on every frame.
    logged_fit: Option<FitKey>,
    decoder: Option<Decoder>,
    /// A message taken from the channel before the pump was ready for it (`start` waits for the first one).
    peeked: Option<Message>,
    /// Counts the decoders started, for tests.
    generation: u32,
    paused: bool,
    /// What was left of the shown frame's delay when the pause began.
    remaining: Option<Duration>,
    /// When the shown frame's delay ends.
    due: Option<Instant>,
    /// The index of the frame on screen in the current pass.
    shown: Option<usize>,
    /// The next frame replaces the picture on screen without moving the timer (a decoder restarted at the shown frame
    /// for a new size).
    resync: bool,
    /// The animation ended and loop is off: the last frame stays.
    finished: bool,
    /// The file is a single frame: it was shown once, no timer runs and loop has nothing to restart.
    still: bool,
    /// The decoder gave up: the last frame stays.
    failed: bool,
    /// Whether the last present failed, so a streak is logged once.
    present_failing: bool,
    /// The group's time base as Unix milliseconds; `None` for a wallpaper on one monitor (D-043).
    epoch: Option<u64>,
    /// An instant and the Unix time it was, so the Unix time at any later instant is that plus the time since: the time
    /// base is compared with the pump's own clock, and a test moves both.
    anchor: (Instant, u64),
    /// The delays of one whole pass from frame 0, as far as they are known.
    delays: Vec<u32>,
    /// Whether `delays` holds a whole pass.
    delays_known: bool,
    /// Whether the pass under way started at frame 0 and its delays are being collected.
    recording: bool,
    /// The frame a time-base jump goes to: the next frame shown is it, and its delay is what is left of it.
    jump_to: Option<usize>,
}

impl GifPump {
    /// Builds the pump over a presenter, with the first decoder already running on `source`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        presenter: Box<dyn Presenter>,
        timers: Box<dyn TimerPort>,
        source: Source,
        bound: (u32, u32),
        stretch: Stretch,
        looping: bool,
        wake: Wake,
        on_playing: Arc<dyn Fn() + Send + Sync>,
    ) -> (Arc<GifEngine>, Self) {
        let shared = Arc::new(Shared {
            looping: AtomicBool::new(looping),
            paused: AtomicBool::new(false),
        });
        let (sender, receiver) = bounded(QUEUE_CAPACITY);
        let engine = Arc::new(GifEngine {
            commands: sender,
            wake: Arc::clone(&wake),
            shared: Arc::clone(&shared),
        });
        let mut pump = Self {
            presenter,
            timers,
            commands: receiver,
            shared,
            wake,
            on_playing: Some(on_playing),
            source,
            bound,
            stretch,
            frame_size: None,
            logged_fit: None,
            decoder: None,
            peeked: None,
            generation: 0,
            paused: false,
            remaining: None,
            due: None,
            shown: None,
            resync: false,
            finished: false,
            still: false,
            failed: false,
            present_failing: false,
            epoch: None,
            anchor: (Instant::now(), unix_now_ms()),
            delays: Vec::new(),
            delays_known: false,
            recording: true,
            jump_to: None,
        };
        pump.restart(0);
        (engine, pump)
    }

    /// Starts a decoder at frame `skip` for the current bound; the old one stops first.
    fn restart(&mut self, skip: usize) {
        self.decoder = None;
        self.peeked = None;
        self.generation += 1;
        self.finished = false;
        self.still = false;
        self.failed = false;
        // Only a pass that starts at frame 0 can give the delays of the whole animation.
        self.recording = skip == 0 && !self.delays_known;
        if self.recording {
            self.delays.clear();
        }
        self.decoder = Some(Decoder::spawn(
            self.source.clone(),
            self.bound,
            fit::scaling(self.stretch),
            skip,
            Arc::clone(&self.shared),
            Arc::clone(&self.wake),
        ));
    }

    /// How many decoders were started so far (a move of the surface starts none, a new size one).
    #[must_use]
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Waits for the first message of the decoder: a failure here is a start failure, so a damaged file never
    /// becomes a wallpaper that shows nothing.
    ///
    /// # Errors
    /// The decoder's reason when it could not produce a first frame, or a timeout sentence.
    pub fn wait_first_frame(&mut self, timeout: Duration) -> Result<(), String> {
        let Some(receiver) = self.decoder.as_ref().and_then(Decoder::receiver) else {
            return Err("the GIF decoder did not start".to_owned());
        };
        match receiver.recv_timeout(timeout) {
            Ok(Message::Failed(reason)) => Err(reason),
            Ok(message) => {
                self.peeked = Some(message);
                Ok(())
            }
            Err(_) => Err("decoding the first frame took too long".to_owned()),
        }
    }

    fn next_message(&mut self) -> Result<Message, TryRecvError> {
        if let Some(message) = self.peeked.take() {
            return Ok(message);
        }
        match self.decoder.as_ref().and_then(Decoder::receiver) {
            Some(receiver) => receiver.try_recv(),
            None => Err(TryRecvError::Disconnected),
        }
    }

    fn arm(&self, wait: Duration) {
        let ms = u32::try_from(wait.as_millis()).unwrap_or(u32::MAX).max(1);
        self.timers.set(GIF_TIMER, ms);
    }

    /// The Unix time at `now` by the pump's own clock.
    fn wall(&self, now: Instant) -> u64 {
        let since = now.saturating_duration_since(self.anchor.0);
        self.anchor
            .1
            .saturating_add(u64::try_from(since.as_millis()).unwrap_or(u64::MAX))
    }

    /// Where the time base puts the animation now: the frame to jump to when a loop just wrapped, or `None` when there
    /// is no time base, the delays of a whole pass are not known yet, or the frame the wrap came to is at most one frame
    /// from the one the time base gives (the renderers of the group are in step).
    fn sync_target(&self, now: Instant) -> Option<usize> {
        let epoch = self.epoch?;
        if !self.delays_known {
            return None;
        }
        let frames = self.delays.len();
        let (index, _) = frame_at(&self.delays, self.wall(now).saturating_sub(epoch), true);
        let distance = index.min(frames - index);
        (distance > 1).then_some(index)
    }

    /// A loop wrapped: learns the delays of the pass that ended when it was recorded whole, logs the wrap and returns
    /// the frame the time base gives when it is more than one frame away.
    fn note_wrap(&mut self, frames: usize, now: Instant) -> Option<usize> {
        if self.recording && !self.delays_known && frames > 0 && self.delays.len() == frames {
            self.delays_known = true;
        }
        // The next pass starts at frame 0, unless a jump below says otherwise.
        self.recording = !self.delays_known;
        if self.recording {
            self.delays.clear();
        }
        tracing::debug!(unix_ms = self.wall(now), "gif loop wrapped");
        self.sync_target(now)
    }

    fn show(&mut self, frame: DecodedFrame, now: Instant) {
        if self.recording && !self.resync {
            self.delays.push(frame.delay_ms);
        }
        let delay = Duration::from_millis(u64::from(frame.delay_ms));
        self.frame_size = Some((frame.width, frame.height));
        match self.presenter.present(frame) {
            Ok(()) => self.present_failing = false,
            Err(PresentError::DeviceLost) => tracing::debug!("the render target was lost while presenting a GIF frame"),
            Err(reason) => {
                if !self.present_failing {
                    tracing::warn!(%reason, "cannot present a GIF frame");
                }
                self.present_failing = true;
            }
        }
        self.note_fit();
        if !self.resync {
            self.shown = Some(self.shown.map_or(0, |index| index + 1));
            // The previous due time keeps the pace (a timer that fires late does not slow the animation), unless the
            // pump fell more than a whole delay behind.
            let base = match self.due {
                Some(due) if now.saturating_duration_since(due) <= delay => due,
                _ => now,
            };
            self.due = Some(base + delay);
            // The frame a time-base jump came to stays for what the time base has left of it.
            if let (Some(target), Some(epoch)) = (self.jump_to.take(), self.epoch)
                && self.delays_known
            {
                let (index, left) = frame_at(&self.delays, self.wall(now).saturating_sub(epoch), true);
                if index == target {
                    self.due = Some(now + Duration::from_millis(u64::from(left)));
                }
            }
        }
        self.resync = false;
        if let Some(on_playing) = self.on_playing.take() {
            tracing::info!(frame_ms = delay.as_millis() as u64, "gif first frame presented");
            on_playing();
        }
    }

    /// Shows the only frame of the file and stops: no timer is armed and nothing follows it.
    fn show_still(&mut self, frame: DecodedFrame, now: Instant) {
        self.show(frame, now);
        self.due = None;
        self.remaining = None;
        self.finished = true;
        self.still = true;
        self.decoder = None;
        self.timers.kill(GIF_TIMER);
    }

    /// Logs the rectangles the shown frame is drawn with when the mode or a size changed since the last line.
    fn note_fit(&mut self) {
        let Some(frame) = self.frame_size else {
            return;
        };
        let key = (self.stretch, frame, self.bound);
        if self.logged_fit == Some(key) {
            return;
        }
        self.logged_fit = Some(key);
        let (dest, src) = fit::fit(self.stretch, frame, self.bound);
        tracing::debug!("gif fit stretch={:?} dest={dest:?} src={src:?}", self.stretch);
    }

    /// Does what is due at `now`: shows the next frame when the shown one has stayed long enough, and arms the timer
    /// for the rest of its delay. The renderer calls it on a wake and on the frame timer; tests call it with a clock
    /// of their own.
    pub fn tick_at(&mut self, now: Instant) {
        // A decoder restarted for a new size replaces the picture first, also while paused.
        if self.resync {
            match self.next_message() {
                Ok(Message::Frame(frame)) => self.show(frame, now),
                Ok(Message::Still(frame)) => self.show_still(frame, now),
                Ok(other) => {
                    // Nothing to redraw at that index any more: the new decoder carries on from its own state.
                    self.resync = false;
                    self.peeked = Some(other);
                }
                Err(_) => return,
            }
        }
        if self.paused {
            self.timers.kill(GIF_TIMER);
            return;
        }
        if self.finished || self.failed {
            self.timers.kill(GIF_TIMER);
            return;
        }
        if let Some(due) = self.due
            && now < due
        {
            self.arm(due - now);
            return;
        }
        loop {
            match self.next_message() {
                Ok(Message::Frame(frame)) => {
                    self.show(frame, now);
                    if let Some(due) = self.due {
                        self.arm(due.saturating_duration_since(now));
                    }
                    return;
                }
                Ok(Message::Still(frame)) => {
                    self.show_still(frame, now);
                    return;
                }
                Ok(Message::PassEnd { continues }) => {
                    let frames = self.shown.map_or(0, |index| index + 1);
                    tracing::debug!("gif loop completed frames={frames}");
                    if self.shared.looping.load(Ordering::SeqCst) {
                        self.shown = None;
                        match self.note_wrap(frames, now) {
                            Some(target) => {
                                // The group's time base puts the animation elsewhere: the decoder starts at that frame.
                                self.restart(target);
                                self.shown = target.checked_sub(1);
                                self.jump_to = Some(target);
                            }
                            // Loop was turned on after the decoder stopped: start it again from frame 0.
                            None if !continues => self.restart(0),
                            None => {}
                        }
                    } else {
                        self.finished = true;
                        self.decoder = None;
                        self.timers.kill(GIF_TIMER);
                        return;
                    }
                }
                Ok(Message::Failed(reason)) => {
                    tracing::error!(%reason, "the GIF decoder gave up, the last frame stays");
                    self.failed = true;
                    self.timers.kill(GIF_TIMER);
                    return;
                }
                Err(TryRecvError::Empty) => {
                    // The decode thread wakes the loop when it has a frame.
                    self.timers.kill(GIF_TIMER);
                    return;
                }
                Err(TryRecvError::Disconnected) => {
                    self.failed = true;
                    self.timers.kill(GIF_TIMER);
                    return;
                }
            }
        }
    }

    /// Applies what the IPC threads queued.
    pub fn drain_at(&mut self, now: Instant) {
        while let Ok(command) = self.commands.try_recv() {
            tracing::debug!(?command, "queued command reached the GIF thread");
            match command {
                Command::Pause { paused, .. } => self.pause(paused, now),
                Command::Loop(looping) => {
                    if looping && self.finished && !self.still {
                        // set_loop(true) after the last frame: start again from frame 0.
                        self.shown = None;
                        self.due = None;
                        self.restart(0);
                    }
                }
                Command::Stretch(stretch) => self.apply_stretch(stretch),
                Command::TimeBase(epoch) => {
                    self.epoch = Some(epoch);
                    tracing::debug!(epoch, "gif time base set");
                }
                Command::Load(path) => {
                    self.source = Source::Path(path);
                    self.delays_known = false;
                    self.jump_to = None;
                    self.shown = None;
                    self.due = None;
                    self.remaining = None;
                    self.resync = false;
                    self.restart(0);
                }
            }
        }
    }

    fn pause(&mut self, paused: bool, now: Instant) {
        if paused == self.paused {
            return;
        }
        self.paused = paused;
        if paused {
            self.remaining = self.due.map(|due| due.saturating_duration_since(now));
            self.due = None;
            self.timers.kill(GIF_TIMER);
        } else {
            self.due = self.remaining.take().map(|left| now + left);
        }
    }

    /// A new stretch mode: the presenter draws the frame on screen again with the new rectangles, and the decoder is
    /// reopened at that frame when the mode needs frames prepared another way (Fill and Uniform share one).
    fn apply_stretch(&mut self, stretch: Stretch) {
        if stretch == self.stretch {
            return;
        }
        let reprepare = fit::scaling(stretch) != fit::scaling(self.stretch);
        self.stretch = stretch;
        self.presenter.set_stretch(stretch);
        self.note_fit();
        if reprepare {
            self.reopen_at_shown();
        }
    }

    /// Reopens the decoder at the frame on screen so that it comes again prepared for the new bound or mode.
    fn reopen_at_shown(&mut self) {
        match self.shown {
            Some(index) if !self.failed => {
                self.restart(index);
                self.resync = true;
                // The frame on screen is drawn again: it is not a new delay of the pass.
                self.recording = false;
                if self.finished {
                    // Leaving the finished state only to redraw: the pass ends again right after the frame.
                    self.finished = false;
                }
            }
            _ => self.restart(0),
        }
    }

    /// The surface changed to `rect`: the window and the render target follow it, and a new size re-opens the decoder
    /// at the frame on screen.
    pub fn resize_to(&mut self, rect: [i32; 4]) {
        let size = surface_size(rect);
        self.presenter.resize(size.0, size.1);
        if size == self.bound {
            return;
        }
        self.bound = size;
        self.note_fit();
        self.reopen_at_shown();
    }
}

impl SurfacePump for GifPump {
    fn on_wake(&mut self) {
        let now = Instant::now();
        self.drain_at(now);
        self.tick_at(now);
    }

    fn on_timer(&mut self, id: u32) -> bool {
        if id != GIF_TIMER {
            return false;
        }
        self.tick_at(Instant::now());
        true
    }

    fn set_rect(&mut self, rect: [i32; 4]) {
        self.resize_to(rect);
    }

    fn close(self: Box<Self>) {
        self.timers.kill(GIF_TIMER);
        // The decoder stops and joins, then the presenter destroys its window (a child of the surface).
        drop(self);
    }
}

/// The width and height of the surface rectangle (left, top, right, bottom), never below one pixel.
#[must_use]
pub fn surface_size(rect: [i32; 4]) -> (u32, u32) {
    let side = |low: i32, high: i32| u32::try_from(high.saturating_sub(low).max(1)).unwrap_or(1);
    (side(rect[0], rect[2]), side(rect[1], rect[3]))
}

/// Starts the built-in GIF renderer on the calling thread, which must own the surface `wid` and run its message loop.
///
/// # Errors
/// [`EngineError::Unsupported`] on a system without Direct2D, [`EngineError::Start`] when the presenter cannot be
/// created or the file has no playable first frame.
pub fn start(wid: u64, assignment: &Assignment, hooks: SurfaceHooks) -> Result<Started, EngineError> {
    imp::start(wid, assignment, hooks)
}

#[cfg(windows)]
mod imp {
    use super::*;

    pub(super) fn start(wid: u64, assignment: &Assignment, hooks: SurfaceHooks) -> Result<Started, EngineError> {
        let SurfaceHooks {
            proxy,
            timers,
            on_playing,
            on_data_wanted: _,
        } = hooks;
        let bound = surface_size(assignment.rect);
        let stretch = assignment.settings.stretch;
        let presenter = crate::present::open(wid, bound.0, bound.1, stretch).map_err(|reason| {
            tracing::error!(%reason, "cannot create the GIF presenter");
            EngineError::Start { reason }
        })?;
        tracing::info!("gif presenter={} stretch={stretch:?}", presenter.name());
        let source = Source::Path(PathBuf::from(&assignment.source));
        let wake: Wake = Arc::new(move || proxy.wake());
        let (engine, mut pump) = GifPump::new(
            Box::new(presenter),
            Box::new(timers),
            source,
            bound,
            stretch,
            assignment.settings.looping,
            wake,
            on_playing,
        );
        if let Err(reason) = pump.wait_first_frame(FIRST_FRAME_WAIT) {
            tracing::error!(%reason, source = %assignment.source, "the animation cannot be played");
            Box::new(pump).close();
            return Err(EngineError::Start {
                reason: format!("The animation cannot be played: {reason}."),
            });
        }
        // The first frame is waiting: the first tick shows it even before the first wake arrives.
        pump.timers.set(GIF_TIMER, 1);
        Ok(Started {
            engine,
            pump: Some(Box::new(pump)),
        })
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;

    pub(super) fn start(_wid: u64, _assignment: &Assignment, _hooks: SurfaceHooks) -> Result<Started, EngineError> {
        Err(EngineError::Unsupported {
            what: "The built-in GIF renderer on this system".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Mutex,
        atomic::{AtomicU32, AtomicUsize},
    };

    use super::*;
    use crate::decode::fixtures::{frame_number, gif_bytes};

    #[derive(Default)]
    struct Recorder {
        /// The frame numbers and sizes shown, in order.
        shown: Mutex<Vec<(u32, u32, u32)>>,
        resizes: Mutex<Vec<(u32, u32)>>,
        stretches: Mutex<Vec<Stretch>>,
    }

    struct FakePresenter(Arc<Recorder>);

    impl Presenter for FakePresenter {
        fn present(&mut self, frame: DecodedFrame) -> Result<(), PresentError> {
            self.0
                .shown
                .lock()
                .expect("lock")
                .push((frame_number(&frame.pixels), frame.width, frame.height));
            Ok(())
        }

        fn resize(&mut self, width: u32, height: u32) {
            self.0.resizes.lock().expect("lock").push((width, height));
        }

        fn set_stretch(&mut self, stretch: Stretch) {
            self.0.stretches.lock().expect("lock").push(stretch);
        }

        fn name(&self) -> &'static str {
            "Fake"
        }
    }

    #[derive(Default)]
    struct TimerLog {
        armed: Mutex<Vec<u32>>,
        kills: AtomicUsize,
    }

    struct FakeTimers(Arc<TimerLog>);

    impl TimerPort for FakeTimers {
        fn set(&self, _id: u32, ms: u32) {
            self.0.armed.lock().expect("lock").push(ms);
        }

        fn kill(&self, _id: u32) {
            self.0.kills.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// What the pump's Unix clock reads at the rig's first tick.
    const UNIX_BASE: u64 = 1_000_000;

    struct Rig {
        engine: Arc<GifEngine>,
        pump: GifPump,
        recorder: Arc<Recorder>,
        timers: Arc<TimerLog>,
        playing: Arc<AtomicU32>,
        clock: Instant,
    }

    impl Rig {
        fn new(frames: usize, size: (u32, u32), bound: (u32, u32), looping: bool) -> Self {
            Self::with_stretch(frames, size, bound, looping, Stretch::Fill)
        }

        fn with_stretch(frames: usize, size: (u32, u32), bound: (u32, u32), looping: bool, stretch: Stretch) -> Self {
            Self::with_delays(&vec![10; frames], size, bound, looping, stretch)
        }

        fn with_delays(delays: &[u32], size: (u32, u32), bound: (u32, u32), looping: bool, stretch: Stretch) -> Self {
            let recorder = Arc::new(Recorder::default());
            let timers = Arc::new(TimerLog::default());
            let playing = Arc::new(AtomicU32::new(0));
            let counter = Arc::clone(&playing);
            let bytes = gif_bytes(size.0, size.1, delays);
            let (engine, mut pump) = GifPump::new(
                Box::new(FakePresenter(Arc::clone(&recorder))),
                Box::new(FakeTimers(Arc::clone(&timers))),
                Source::Memory(Arc::from(bytes)),
                bound,
                stretch,
                looping,
                Arc::new(|| {}),
                Arc::new(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                }),
            );
            let clock = Instant::now();
            // The pump's Unix clock reads 1_000_000 ms at the rig's first tick.
            pump.anchor = (clock, UNIX_BASE);
            Self {
                engine,
                pump,
                recorder,
                timers,
                playing,
                clock,
            }
        }

        fn numbers(&self) -> Vec<u32> {
            self.recorder
                .shown
                .lock()
                .expect("lock")
                .iter()
                .map(|shown| shown.0)
                .collect()
        }

        /// Ticks with a clock that moves 50 ms per round (more than any frame delay here) until `done`.
        fn run_until(&mut self, done: impl Fn(&Self) -> bool) {
            for _ in 0..600 {
                self.pump.drain_at(self.clock);
                self.pump.tick_at(self.clock);
                if done(self) {
                    return;
                }
                self.clock += Duration::from_millis(50);
                thread::sleep(Duration::from_millis(2));
            }
            panic!("the condition was not reached, frames shown: {:?}", self.numbers());
        }

        /// Ticks a few rounds more and lets the decoder run, to see that nothing else is shown.
        fn settle(&mut self) {
            for _ in 0..30 {
                self.pump.drain_at(self.clock);
                self.pump.tick_at(self.clock);
                self.clock += Duration::from_millis(50);
                thread::sleep(Duration::from_millis(2));
            }
        }
    }

    #[test]
    fn the_first_frame_is_shown_once_and_reports_playing_once() {
        let mut rig = Rig::new(3, (16, 9), (16, 9), true);
        rig.run_until(|rig| rig.numbers().len() >= 7);
        assert_eq!(rig.playing.load(Ordering::SeqCst), 1);
        assert_eq!(
            &rig.numbers()[..7],
            [0, 1, 2, 0, 1, 2, 0],
            "a looping animation wraps in order"
        );
        assert!(rig.timers.armed.lock().expect("lock").iter().all(|ms| *ms >= 1));
    }

    #[test]
    fn loop_off_stops_on_the_last_frame_and_loop_on_continues_from_frame_zero() {
        let mut rig = Rig::new(3, (16, 9), (16, 9), false);
        rig.engine.set_loop(false).expect("loop");
        rig.run_until(|rig| rig.numbers().len() >= 3);
        rig.settle();
        assert_eq!(rig.numbers(), [0, 1, 2], "the last frame stays, nothing more is shown");
        assert!(rig.pump.finished);

        rig.engine.set_loop(true).expect("loop");
        rig.run_until(|rig| rig.numbers().len() >= 5);
        assert_eq!(&rig.numbers()[3..5], [0, 1], "loop on starts again from frame 0");
        assert!(!rig.pump.finished);
    }

    #[test]
    fn a_single_frame_is_shown_once_and_arms_no_frame_timer_whatever_loop_says() {
        for looping in [true, false] {
            let mut rig = Rig::new(1, (16, 9), (16, 9), looping);
            rig.run_until(|rig| !rig.numbers().is_empty());
            rig.settle();
            assert_eq!(rig.numbers(), [0], "loop {looping}: shown once and never again");
            assert!(
                rig.timers.armed.lock().expect("lock").is_empty(),
                "loop {looping}: no frame timer was armed"
            );
            assert_eq!(rig.playing.load(Ordering::SeqCst), 1);
            assert!(rig.pump.finished && rig.pump.still);

            rig.engine.set_loop(!looping).expect("loop");
            rig.settle();
            assert_eq!(rig.numbers(), [0], "turning loop around redraws nothing");
            assert!(rig.timers.armed.lock().expect("lock").is_empty());
        }
    }

    #[test]
    fn a_single_frame_is_redrawn_once_for_a_new_size_and_still_starts_no_timer() {
        let mut rig = Rig::new(1, (32, 18), (32, 18), true);
        rig.run_until(|rig| !rig.numbers().is_empty());
        rig.pump.resize_to([0, 0, 16, 9]);
        rig.run_until(|rig| rig.numbers().len() >= 2);
        rig.settle();
        assert_eq!(rig.numbers().len(), 2, "one more draw for the new size and no more");
        assert_eq!(
            rig.recorder.shown.lock().expect("lock")[1].1,
            16,
            "prepared for the new surface"
        );
        assert!(rig.timers.armed.lock().expect("lock").is_empty());
    }

    #[test]
    fn a_pause_keeps_the_frame_and_resume_continues_from_it() {
        for level in [PauseLevel::Light, PauseLevel::Heavy] {
            let mut rig = Rig::new(4, (16, 9), (16, 9), true);
            rig.run_until(|rig| rig.numbers().len() >= 2);
            rig.engine.set_paused(true, level).expect("pause");
            assert!(rig.engine.paused());
            rig.settle();
            let paused_at = rig.numbers();
            assert_eq!(paused_at.len(), 2, "{level:?}: no frame is shown while paused");
            rig.clock += Duration::from_secs(10);
            rig.settle();
            assert_eq!(rig.numbers(), paused_at, "{level:?}: a long pause shows nothing either");

            rig.engine.set_paused(false, level).expect("resume");
            rig.run_until(|rig| rig.numbers().len() >= 4);
            assert_eq!(
                &rig.numbers()[..4],
                [0, 1, 2, 3],
                "{level:?}: resume goes on with the next frame"
            );
        }
    }

    #[test]
    fn a_smaller_surface_reopens_the_decoder_at_the_frame_on_screen_and_a_move_does_not() {
        let mut rig = Rig::new(4, (64, 36), (64, 36), true);
        rig.run_until(|rig| rig.numbers().len() >= 3);
        let generation = rig.pump.generation();

        // A move of the surface keeps the size: no new decoder.
        rig.pump.resize_to([100, 50, 164, 86]);
        assert_eq!(rig.pump.generation(), generation);

        // A smaller surface: the window follows, the decoder is reopened and frame 2 comes again, smaller.
        rig.pump.resize_to([0, 0, 32, 18]);
        assert_eq!(rig.pump.generation(), generation + 1);
        assert_eq!(rig.recorder.resizes.lock().expect("lock").last(), Some(&(32, 18)));
        let before = rig.recorder.shown.lock().expect("lock").len();
        rig.run_until(|rig| rig.recorder.shown.lock().expect("lock").len() > before);
        let redrawn = rig.recorder.shown.lock().expect("lock")[before];
        assert_eq!(
            redrawn,
            (2, 32, 18),
            "the frame on screen is drawn again at the new size"
        );
        // And the animation goes on with frame 3, not frame 0.
        rig.run_until(|rig| rig.recorder.shown.lock().expect("lock").len() > before + 1);
        assert_eq!(rig.recorder.shown.lock().expect("lock")[before + 1].0, 3);
    }

    #[test]
    fn the_stretch_at_start_decides_how_the_first_frame_is_prepared() {
        // A 100 x 100 animation on a 40 x 20 surface: Fill and Uniform fit it, UniformToFill keeps it covering the
        // surface and None keeps only the centre that fits.
        for (stretch, expected) in [
            (Stretch::Fill, (20, 20)),
            (Stretch::Uniform, (20, 20)),
            (Stretch::UniformToFill, (40, 40)),
            (Stretch::None, (40, 20)),
        ] {
            let mut rig = Rig::with_stretch(2, (100, 100), (40, 20), true, stretch);
            rig.run_until(|rig| !rig.numbers().is_empty());
            let shown = rig.recorder.shown.lock().expect("lock")[0];
            assert_eq!((shown.1, shown.2), expected, "{stretch:?}");
        }
    }

    #[test]
    fn a_new_stretch_reaches_the_presenter_and_reopens_the_decoder_only_when_frames_are_prepared_differently() {
        let mut rig = Rig::with_stretch(4, (100, 100), (40, 20), true, Stretch::Fill);
        rig.run_until(|rig| rig.numbers().len() >= 2);
        let generation = rig.pump.generation();

        // Fill to Uniform: the same frames, drawn with other rectangles.
        rig.engine.set_stretch(Stretch::Uniform).expect("stretch");
        rig.pump.drain_at(rig.clock);
        assert_eq!(
            rig.recorder.stretches.lock().expect("lock").as_slice(),
            [Stretch::Uniform]
        );
        assert_eq!(rig.pump.generation(), generation);

        // The same mode again changes nothing.
        rig.engine.set_stretch(Stretch::Uniform).expect("stretch");
        rig.pump.drain_at(rig.clock);
        assert_eq!(rig.recorder.stretches.lock().expect("lock").len(), 1);

        // Uniform to UniformToFill: frames are prepared to cover, so the decoder reopens at the frame on screen.
        let before = rig.recorder.shown.lock().expect("lock").len();
        rig.engine.set_stretch(Stretch::UniformToFill).expect("stretch");
        rig.pump.drain_at(rig.clock);
        assert_eq!(rig.pump.generation(), generation + 1);
        rig.run_until(|rig| rig.recorder.shown.lock().expect("lock").len() > before);
        let redrawn = rig.recorder.shown.lock().expect("lock")[before];
        assert_eq!(redrawn, (1, 40, 40), "frame 1 is drawn again, prepared to cover");
    }

    #[test]
    fn a_damaged_file_fails_the_first_frame_wait_with_a_reason() {
        let recorder = Arc::new(Recorder::default());
        let (_engine, mut pump) = GifPump::new(
            Box::new(FakePresenter(recorder)),
            Box::new(FakeTimers(Arc::default())),
            Source::Memory(Arc::from(b"not a gif".to_vec())),
            (8, 8),
            Stretch::Fill,
            true,
            Arc::new(|| {}),
            Arc::new(|| {}),
        );
        let reason = pump
            .wait_first_frame(Duration::from_secs(5))
            .expect_err("not an animation");
        assert!(reason.contains("not a GIF"), "{reason}");
    }

    #[test]
    fn the_handle_has_no_clock_and_ignores_volume_and_audio() {
        let rig = Rig::new(2, (8, 8), (8, 8), true);
        assert!(rig.engine.clock().is_none());
        assert_eq!(rig.engine.name(), "BuiltInGif");
        assert!(rig.engine.set_volume(10).is_ok());
        assert!(rig.engine.set_audio(true).is_ok());
    }

    #[test]
    fn frame_at_gives_the_frame_and_what_is_left_of_it() {
        assert_eq!(frame_at(&[50; 24], 0, true), (0, 50));
        assert_eq!(frame_at(&[50; 24], 49, true), (0, 1));
        assert_eq!(frame_at(&[50; 24], 50, true), (1, 50));
        assert_eq!(frame_at(&[50; 24], 1210, true), (0, 40), "the loop wraps after 1200 ms");
        assert_eq!(
            frame_at(&[50; 24], 1_200_000_000_010, true),
            (0, 40),
            "a long time is no problem"
        );
        assert_eq!(
            frame_at(&[0, 50], 120, true),
            (1, 30),
            "a delay of 0 plays at the legacy 100 ms"
        );
    }

    #[test]
    fn frame_at_without_a_loop_keeps_the_last_frame_and_an_empty_list_is_frame_zero() {
        assert_eq!(frame_at(&[50; 4], 199, false), (3, 1));
        assert_eq!(frame_at(&[50; 4], 200, false), (3, 0));
        assert_eq!(frame_at(&[50; 4], 9_999, false), (3, 0));
        assert_eq!(frame_at(&[], 123, true), (0, 0));
        assert_eq!(frame_at(&[], 123, false), (0, 0));
        assert!(
            frame_at(&[0, 0], u64::MAX, true).0 < 2,
            "every delay of 0 still makes a loop"
        );
    }

    /// Plays one whole pass of a four-frame animation of 100 ms frames (a 400 ms loop) with the time base at `epoch`
    /// and returns the numbers shown up to two frames after the wrap. The rig ticks every 50 ms, so the wrap is handled
    /// at 400 ms, or at 450 ms when the decoder thread is a round late: the epochs below are far from a frame boundary
    /// at both.
    fn shown_after_the_first_wrap(epoch: Option<u64>) -> Vec<u32> {
        let mut rig = Rig::with_delays(&[100; 4], (16, 9), (16, 9), true, Stretch::Fill);
        if let Some(epoch) = epoch {
            rig.engine.set_time_base(epoch);
        }
        rig.run_until(|rig| rig.numbers().len() >= 6);
        rig.numbers()
    }

    #[test]
    fn at_a_wrap_the_animation_moves_to_the_frame_the_time_base_gives_when_it_is_more_than_one_frame_away() {
        let numbers = shown_after_the_first_wrap(Some(UNIX_BASE - 220));
        assert_eq!(&numbers[..4], [0, 1, 2, 3]);
        assert_eq!(numbers[4], 2, "620 ms into a 400 ms loop is 220 ms: frame 2");
        assert_eq!(numbers[5], 3, "and it plays on from there");
    }

    #[test]
    fn a_frame_in_step_or_one_frame_away_is_left_alone_in_both_directions() {
        for (epoch, name) in [
            (UNIX_BASE, "on the grid"),
            (UNIX_BASE - 100, "one frame ahead of the grid"),
            (UNIX_BASE + 100, "one frame behind the grid"),
        ] {
            let numbers = shown_after_the_first_wrap(Some(epoch));
            assert_eq!(
                numbers[4], 0,
                "{name}: the loop wraps to frame 0 as it does without a time base"
            );
        }
        assert_eq!(shown_after_the_first_wrap(None)[4], 0, "no time base, no jump");
    }

    #[test]
    fn the_first_pass_has_no_delays_to_jump_by_and_a_later_time_base_acts_at_the_next_wrap() {
        let mut rig = Rig::with_delays(&[100; 4], (16, 9), (16, 9), true, Stretch::Fill);
        rig.run_until(|rig| rig.numbers().len() >= 2);
        rig.engine.set_time_base(UNIX_BASE - 220);
        rig.run_until(|rig| rig.numbers().len() >= 6);
        assert_eq!(
            rig.numbers()[4],
            2,
            "the wrap after the time base arrived moves to frame 2"
        );
    }

    #[test]
    fn a_single_frame_ignores_the_time_base() {
        let mut rig = Rig::with_delays(&[50], (16, 9), (16, 9), true, Stretch::Fill);
        rig.engine.set_time_base(UNIX_BASE - 100);
        rig.run_until(|rig| !rig.numbers().is_empty());
        rig.settle();
        assert_eq!(
            rig.numbers(),
            [0],
            "a one-frame animation is shown once whatever the time base says"
        );
        assert!(rig.pump.finished && rig.pump.still);
    }

    #[test]
    fn the_surface_size_is_never_below_one_pixel() {
        assert_eq!(surface_size([0, 0, 1920, 1080]), (1920, 1080));
        assert_eq!(surface_size([10, 10, 10, 10]), (1, 1));
        assert_eq!(surface_size([-1920, 0, 0, 1080]), (1920, 1080));
    }
}
