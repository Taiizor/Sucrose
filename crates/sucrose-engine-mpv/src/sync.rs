//! The follower's drift correction (D-02): given the master's clock and the follower's own position, decide whether
//! to leave playback alone, change its speed or seek. Pure, so the cases are tested without a player.
//!
//! The master publishes a [ClockTick]; a follower extrapolates it to "now" with the shared system clock, compares
//! that with its own position and answers with a [SyncAction]. [Follower] is the controller around it: a band of one
//! video frame with release near zero, a proportional speed correction and an integral trim for a master or follower
//! that does not run at exactly real time (02-20).

use sucrose_ipc::protocol::ClockTick;

/// The drift left alone when the video's frame rate is unknown: one frame at 30 fps.
pub const DEFAULT_FRAME_S: f64 = 1.0 / 30.0;

/// Drift up to this many seconds is corrected by a speed change, beyond it by a seek.
pub const SEEK_THRESHOLD_S: f64 = 0.5;

/// The proportional gain: speed correction per second of drift, so a follower 0.1 s behind runs 5 % faster.
pub const SPEED_GAIN: f64 = 0.5;

/// The integral gain: speed correction per second-squared of accumulated drift. It learns the rate difference
/// between master and follower and keeps it applied inside the band.
pub const INTEGRAL_GAIN: f64 = 0.1;

/// The largest speed correction, as a fraction of the master's speed: 5 % is inaudible (followers play no audio) and
/// invisible, and bounds what a noisy clock can do.
pub const MAX_SPEED_STEP: f64 = 0.05;

/// A correcting follower is released when its drift falls to this fraction of the band, so it does not park at the
/// band's edge.
pub const RELEASE_FRACTION: f64 = 0.25;

/// The integral only accumulates while the drift is within this many seconds; a bigger drift is the proportional
/// term's job and would wind the integral up.
pub const INTEGRAL_ZONE_S: f64 = 0.1;

/// The integral does not accumulate while the drift shrinks faster than this fraction of the previous reading per
/// tick: that is the proportional term closing in on its own, not a rate difference, and counting it would overshoot.
pub const CLOSING_FRACTION: f64 = 0.75;

/// A speed that differs from the last one sent by less than this is not sent again (0.1 %).
pub const MIN_SPEED_CHANGE: f64 = 0.001;

/// The longest time between two readings that counts towards the integral, in seconds.
const MAX_INTEGRATION_S: f64 = 2.0;

/// The frame rates a video can plausibly have; anything else is a bad reading.
const FPS_RANGE: std::ops::RangeInclusive<f64> = 5.0..=240.0;

/// What a follower does about its drift.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncAction {
    /// Leave playback alone.
    None,
    /// Play at this speed, the master's speed corrected by at most 5 %.
    Speed(f64),
    /// Jump to this position in seconds, exactly.
    Seek(f64),
}

/// The drift left alone for a video of `fps` frames per second: one frame of it. An unknown or implausible rate (outside 5
/// to 240) gives [DEFAULT_FRAME_S].
#[must_use]
pub fn frame_period(fps: Option<f64>) -> f64 {
    match fps {
        Some(rate) if FPS_RANGE.contains(&rate) => 1.0 / rate,
        _ => DEFAULT_FRAME_S,
    }
}

/// Classifies the follower at `local_pos_s` against the master's `tick`, read at `now_ns` on the same clock as
/// `tick.stamp_ns`: [SyncAction::None] inside the band of `frame_s`, [SyncAction::Seek] beyond [SEEK_THRESHOLD_S], otherwise
/// a [SyncAction::Speed] in the direction that closes the drift (at the full [MAX_SPEED_STEP]; [Follower] chooses the
/// actual amount). `duration_s` is the file length when it loops, so a wrap is not mistaken for drift. A paused master
/// gives nothing to chase.
#[must_use]
pub fn correct(tick: &ClockTick, now_ns: u64, local_pos_s: f64, duration_s: Option<f64>, frame_s: f64) -> SyncAction {
    if tick.paused {
        return SyncAction::None;
    }
    let expected = expected_position(tick, now_ns, duration_s);
    let drift = signed_drift(local_pos_s, expected, duration_s);
    if drift.abs() <= frame_s {
        SyncAction::None
    } else if drift.abs() <= SEEK_THRESHOLD_S {
        // Ahead of the master: slow down. Behind it: speed up.
        let factor = if drift > 0.0 {
            1.0 - MAX_SPEED_STEP
        } else {
            1.0 + MAX_SPEED_STEP
        };
        SyncAction::Speed(tick.speed * factor)
    } else {
        SyncAction::Seek(expected)
    }
}

/// Where the master is at `now_ns`: its position when the tick was taken plus the time since, at its speed, wrapped
/// into the file when it loops.
fn expected_position(tick: &ClockTick, now_ns: u64, duration_s: Option<f64>) -> f64 {
    let elapsed_s = seconds(now_ns.saturating_sub(tick.stamp_ns));
    let expected = tick.position_s + elapsed_s * tick.speed;
    match duration_s.filter(|length| *length > 0.0) {
        Some(length) => expected.rem_euclid(length),
        None => expected,
    }
}

/// `local - expected` in seconds; positive when the follower is ahead. In a loop the distance is the shorter way
/// round the circle, so a follower just before the end and a master just after it are close, not a file apart.
fn signed_drift(local_pos_s: f64, expected_s: f64, duration_s: Option<f64>) -> f64 {
    let drift = local_pos_s - expected_s;
    match duration_s.filter(|length| *length > 0.0) {
        Some(length) => {
            let wrapped = drift.rem_euclid(length);
            if wrapped > length / 2.0 {
                wrapped - length
            } else {
                wrapped
            }
        }
        None => drift,
    }
}

/// Nanoseconds as seconds. The precision lost above 2^53 ns (about 104 days) is far below anything a clock tick needs.
#[allow(clippy::cast_precision_loss)]
fn seconds(ns: u64) -> f64 {
    ns as f64 / 1e9
}

/// Two seconds in nanoseconds: an exact seek needs time to finish, so another one is not issued before that.
pub const MIN_SEEK_GAP_NS: u64 = 2_000_000_000;

/// What a follower did over a number of ticks, for the periodic log line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriftSummary {
    /// How many readings of a running master the numbers cover.
    pub ticks: u32,
    /// The mean of the absolute drift at those readings, in seconds.
    pub mean_abs_s: f64,
    /// The largest absolute drift at those readings, in seconds.
    pub max_abs_s: f64,
    /// How many times the follower's speed was changed.
    pub speed_changes: u32,
    /// How many times the follower seeked.
    pub seeks: u32,
}

/// The sums behind a [DriftSummary].
#[derive(Debug, Default)]
struct Counters {
    ticks: u32,
    sum_abs_s: f64,
    max_abs_s: f64,
    speed_changes: u32,
    seeks: u32,
}

impl Counters {
    fn record(&mut self, drift_s: f64) {
        self.ticks = self.ticks.saturating_add(1);
        self.sum_abs_s += drift_s.abs();
        self.max_abs_s = self.max_abs_s.max(drift_s.abs());
    }
}

/// A follower's controller between two readings: whether it is correcting (with hysteresis: it starts above one
/// frame and stops at a quarter of one), the integral of its drift that learns a rate difference, the last speed it
/// sent, when it last seeked, and the counters of the next [DriftSummary].
#[derive(Debug)]
pub struct Follower {
    correcting: bool,
    integral: f64,
    last_now_ns: Option<u64>,
    last_drift_s: Option<f64>,
    last_speed: f64,
    last_seek_ns: Option<u64>,
    counters: Counters,
}

impl Default for Follower {
    fn default() -> Self {
        Self {
            correcting: false,
            integral: 0.0,
            last_now_ns: None,
            last_drift_s: None,
            // A new player runs at normal speed.
            last_speed: 1.0,
            last_seek_ns: None,
            counters: Counters::default(),
        }
    }
}

impl Follower {
    /// A follower at normal speed that has not corrected or seeked yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Decides what this follower does about the master's `tick` (see [correct] for the arguments), but only returns an
    /// action that changes something. Beyond [SEEK_THRESHOLD_S] it seeks (not again within two seconds). Otherwise
    /// the speed is the master's times `1 - c`, where `c` is [SPEED_GAIN] times the drift while correcting plus
    /// [INTEGRAL_GAIN] times the accumulated drift, limited to [MAX_SPEED_STEP]; it is only sent when it differs
    /// from the last one by [MIN_SPEED_CHANGE]. A paused master gives back the master's speed and clears the
    /// integral.
    pub fn step(
        &mut self,
        tick: &ClockTick,
        now_ns: u64,
        local_pos_s: f64,
        duration_s: Option<f64>,
        frame_s: f64,
    ) -> SyncAction {
        let elapsed_s = self.elapsed_since_last(now_ns);
        if tick.paused {
            self.clear_control();
            return self.speed_if_changed(tick.speed);
        }
        let drift = signed_drift(local_pos_s, expected_position(tick, now_ns, duration_s), duration_s);
        self.counters.record(drift);
        if let SyncAction::Seek(position) = correct(tick, now_ns, local_pos_s, duration_s, frame_s) {
            return self.seek(position, now_ns);
        }
        let magnitude = drift.abs();
        if self.correcting {
            self.correcting = magnitude > frame_s * RELEASE_FRACTION;
        } else {
            self.correcting = magnitude > frame_s;
        }
        let proportional = if self.correcting { SPEED_GAIN * drift } else { 0.0 };
        let closing = self
            .last_drift_s
            .is_some_and(|previous| magnitude < CLOSING_FRACTION * previous.abs());
        self.last_drift_s = Some(drift);
        // Anti-windup: the integral only grows near zero, not while the drift is closing by itself, and not once the
        // output would saturate.
        let candidate = self.integral + drift * elapsed_s;
        let candidate_correction = proportional + INTEGRAL_GAIN * candidate;
        let correction = if magnitude <= INTEGRAL_ZONE_S && !closing && candidate_correction.abs() < MAX_SPEED_STEP {
            self.integral = candidate;
            candidate_correction
        } else {
            (proportional + INTEGRAL_GAIN * self.integral).clamp(-MAX_SPEED_STEP, MAX_SPEED_STEP)
        };
        self.speed_if_changed(tick.speed * (1.0 - correction))
    }

    /// The mean and largest drift, the speed changes and the seeks since the last call, then starts counting anew.
    /// None when no reading of a running master came in.
    pub fn take_summary(&mut self) -> Option<DriftSummary> {
        let counters = std::mem::take(&mut self.counters);
        (counters.ticks > 0).then(|| DriftSummary {
            ticks: counters.ticks,
            mean_abs_s: counters.sum_abs_s / f64::from(counters.ticks),
            max_abs_s: counters.max_abs_s,
            speed_changes: counters.speed_changes,
            seeks: counters.seeks,
        })
    }

    /// Seconds since the previous reading, at most [MAX_INTEGRATION_S]; none for the first.
    fn elapsed_since_last(&mut self, now_ns: u64) -> f64 {
        let elapsed = self
            .last_now_ns
            .map_or(0.0, |last| seconds(now_ns.saturating_sub(last)).min(MAX_INTEGRATION_S));
        self.last_now_ns = Some(now_ns);
        elapsed
    }

    /// Forgets what the controller learned: after a seek or a pause the old drift says nothing about the next.
    fn clear_control(&mut self) {
        self.correcting = false;
        self.integral = 0.0;
        self.last_drift_s = None;
    }

    fn seek(&mut self, position: f64, now_ns: u64) -> SyncAction {
        let too_soon = self
            .last_seek_ns
            .is_some_and(|last| now_ns.saturating_sub(last) <= MIN_SEEK_GAP_NS);
        if too_soon {
            return SyncAction::None;
        }
        self.last_seek_ns = Some(now_ns);
        self.counters.seeks = self.counters.seeks.saturating_add(1);
        self.clear_control();
        SyncAction::Seek(position)
    }

    fn speed_if_changed(&mut self, wanted: f64) -> SyncAction {
        if (wanted - self.last_speed).abs() < MIN_SPEED_CHANGE {
            return SyncAction::None;
        }
        self.last_speed = wanted;
        self.counters.speed_changes = self.counters.speed_changes.saturating_add(1);
        SyncAction::Speed(wanted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: u64 = 1_000_000_000;

    /// One frame at 60 fps.
    const FRAME_60: f64 = 1.0 / 60.0;

    fn tick(position_s: f64, stamp_ns: u64) -> ClockTick {
        ClockTick {
            position_s,
            stamp_ns,
            speed: 1.0,
            paused: false,
            loop_gen: 0,
        }
    }

    fn speed_of(action: SyncAction) -> f64 {
        match action {
            SyncAction::Speed(speed) => speed,
            other => panic!("expected a speed change, got {other:?}"),
        }
    }

    /// What a simulated follower went through.
    struct Run {
        /// The true drift (follower minus master, seconds) at each tick, before the follower reacted.
        drifts: Vec<f64>,
        /// The speed the follower was given at each tick (or kept).
        speeds: Vec<f64>,
        seeks: usize,
    }

    impl Run {
        /// Whether every drift from tick `from` for `count` ticks is within `bound`.
        fn stays_within(&self, bound: f64, from: usize, count: usize) -> bool {
            self.drifts.len() >= from + count && self.drifts[from..from + count].iter().all(|d| d.abs() <= bound)
        }

        fn worst_from(&self, from: usize) -> f64 {
            self.drifts[from..].iter().fold(0.0, |worst, d| d.abs().max(worst))
        }
    }

    /// The master advances at `master_rate` seconds of video per second and sends a tick each second (stamp = now); the
    /// follower starts `start_drift` seconds ahead and advances at `follower_rate` times the speed it was last given;
    /// a seek sets its position.
    fn simulate(master_rate: f64, follower_rate: f64, start_drift: f64, frame_s: f64, ticks: u32) -> Run {
        let mut follower = Follower::new();
        let mut run = Run {
            drifts: Vec::new(),
            speeds: Vec::new(),
            seeks: 0,
        };
        let mut speed = 1.0;
        let mut master = 100.0;
        let mut local = 100.0 + start_drift;
        for k in 0..ticks {
            let now = (u64::from(k) + 1) * SECOND;
            run.drifts.push(local - master);
            let action = follower.step(&tick(master, now), now, local, None, frame_s);
            match action {
                SyncAction::None => {}
                SyncAction::Speed(new) => speed = new,
                SyncAction::Seek(position) => {
                    local = position;
                    run.seeks += 1;
                }
            }
            run.speeds.push(speed);
            local += follower_rate * speed;
            master += master_rate;
        }
        run
    }

    #[test]
    fn the_frame_period_follows_the_video_frame_rate() {
        assert!((frame_period(Some(60.0)) - 1.0 / 60.0).abs() < 1e-12);
        assert!((frame_period(Some(29.97)) - 1.0 / 29.97).abs() < 1e-12);
    }

    #[test]
    fn an_unknown_or_implausible_frame_rate_gives_the_default_period() {
        for fps in [None, Some(0.0), Some(1000.0), Some(f64::NAN), Some(-30.0)] {
            assert!((frame_period(fps) - DEFAULT_FRAME_S).abs() < 1e-12, "{fps:?}");
        }
    }

    #[test]
    fn drift_inside_one_frame_is_left_alone() {
        let action = correct(&tick(10.0, 5 * SECOND), 5 * SECOND, 10.01, None, DEFAULT_FRAME_S);

        assert_eq!(action, SyncAction::None);
    }

    #[test]
    fn a_follower_ahead_by_a_fifth_of_a_second_slows_down() {
        let action = correct(&tick(10.0, 5 * SECOND), 5 * SECOND, 10.2, None, DEFAULT_FRAME_S);

        assert!(speed_of(action) < 1.0, "{action:?}");
    }

    #[test]
    fn a_follower_behind_by_a_fifth_of_a_second_speeds_up() {
        let action = correct(&tick(10.0, 5 * SECOND), 5 * SECOND, 9.8, None, DEFAULT_FRAME_S);

        assert!(speed_of(action) > 1.0, "{action:?}");
    }

    #[test]
    fn a_drift_beyond_half_a_second_seeks_to_the_expected_position() {
        let SyncAction::Seek(target) = correct(&tick(10.0, 5 * SECOND), 5 * SECOND, 10.8, None, DEFAULT_FRAME_S) else {
            panic!("expected a seek");
        };

        assert!((target - 10.0).abs() < 1e-9);
    }

    #[test]
    fn the_expected_position_advances_with_the_time_since_the_tick() {
        // Two seconds after the tick the master is at 12; a follower at 12.0 has no drift, one at 10.0 is far behind.
        let master = tick(10.0, 5 * SECOND);

        assert_eq!(
            correct(&master, 7 * SECOND, 12.0, None, DEFAULT_FRAME_S),
            SyncAction::None
        );
        let SyncAction::Seek(target) = correct(&master, 7 * SECOND, 10.0, None, DEFAULT_FRAME_S) else {
            panic!("expected a seek");
        };
        assert!((target - 12.0).abs() < 1e-9);
    }

    #[test]
    fn across_the_loop_end_the_drift_is_the_circular_distance() {
        // The master wrapped and is at 0.1 s; the follower, 0.05 s before the end of a 30 s file, has not yet: it is
        // 0.15 s behind, not 29.85 s ahead.
        let action = correct(&tick(0.1, 5 * SECOND), 5 * SECOND, 29.95, Some(30.0), DEFAULT_FRAME_S);

        assert!(speed_of(action) > 1.0, "{action:?}");
    }

    #[test]
    fn the_expected_position_wraps_modulo_the_duration() {
        // 0.3 s after a reading at 29.9 s the master is at 0.2 s of a 30 s loop.
        let master = tick(29.9, 5 * SECOND);

        let action = correct(&master, 5 * SECOND + 300_000_000, 0.2, Some(30.0), DEFAULT_FRAME_S);

        assert_eq!(action, SyncAction::None);
    }

    #[test]
    fn a_seek_across_the_loop_end_lands_inside_the_file() {
        let SyncAction::Seek(target) = correct(&tick(29.9, 5 * SECOND), 6 * SECOND, 10.0, Some(30.0), DEFAULT_FRAME_S)
        else {
            panic!("expected a seek");
        };

        assert!((target - 0.9).abs() < 1e-9, "{target}");
    }

    #[test]
    fn a_follower_across_the_loop_end_is_corrected_by_the_circular_distance() {
        let mut follower = Follower::new();

        let action = follower.step(&tick(0.1, 5 * SECOND), 5 * SECOND, 29.95, Some(30.0), FRAME_60);

        assert!(speed_of(action) > 1.0, "{action:?}");
    }

    #[test]
    fn a_follower_returns_to_the_masters_speed_once_it_has_caught_up() {
        let mut follower = Follower::new();
        let master = tick(10.0, 5 * SECOND);

        let behind = follower.step(&master, 5 * SECOND, 9.8, None, DEFAULT_FRAME_S);
        let caught_up = follower.step(&master, 5 * SECOND, 10.0, None, DEFAULT_FRAME_S);
        let after = follower.step(&master, 5 * SECOND, 10.0, None, DEFAULT_FRAME_S);

        assert!(speed_of(behind) > 1.0);
        assert!((speed_of(caught_up) - 1.0).abs() < 1e-9);
        assert_eq!(after, SyncAction::None);
    }

    #[test]
    fn an_unchanged_speed_is_not_sent_again() {
        let mut follower = Follower::new();
        let master = tick(10.0, 5 * SECOND);

        let first = follower.step(&master, 5 * SECOND, 9.8, None, DEFAULT_FRAME_S);
        let second = follower.step(&master, 5 * SECOND, 9.8, None, DEFAULT_FRAME_S);

        assert!(speed_of(first) > 1.0);
        assert_eq!(second, SyncAction::None);
    }

    #[test]
    fn a_second_seek_within_two_seconds_waits() {
        let mut follower = Follower::new();
        let master = tick(10.0, 5 * SECOND);

        let first = follower.step(&master, 5 * SECOND, 12.0, None, DEFAULT_FRAME_S);
        let soon = follower.step(&master, 6 * SECOND, 12.0, None, DEFAULT_FRAME_S);
        let later = follower.step(&master, 7 * SECOND + 1, 15.0, None, DEFAULT_FRAME_S);

        assert!(matches!(first, SyncAction::Seek(_)), "{first:?}");
        assert_eq!(soon, SyncAction::None);
        assert!(matches!(later, SyncAction::Seek(_)), "{later:?}");
    }

    #[test]
    fn a_paused_master_leaves_a_fresh_follower_alone_and_frees_a_corrected_one() {
        let mut fresh = Follower::new();
        let mut corrected = Follower::new();
        let running = tick(10.0, 5 * SECOND);
        let paused = ClockTick {
            paused: true,
            ..running
        };
        corrected.step(&running, 5 * SECOND, 9.8, None, DEFAULT_FRAME_S);

        assert_eq!(
            fresh.step(&paused, 6 * SECOND, 3.0, None, DEFAULT_FRAME_S),
            SyncAction::None
        );
        let freed = corrected.step(&paused, 6 * SECOND, 9.9, None, DEFAULT_FRAME_S);
        assert!((speed_of(freed) - 1.0).abs() < 1e-9, "{freed:?}");
        assert!(corrected.integral.abs() < f64::EPSILON);
    }

    #[test]
    fn a_paused_master_gives_nothing_to_chase() {
        let master = ClockTick {
            paused: true,
            ..tick(10.0, 5 * SECOND)
        };

        assert_eq!(
            correct(&master, 9 * SECOND, 3.0, None, DEFAULT_FRAME_S),
            SyncAction::None
        );
    }

    #[test]
    fn a_follower_converges_at_equal_rates_and_stays() {
        let run = simulate(1.0, 1.0, -0.3, FRAME_60, 70);

        assert!(run.stays_within(FRAME_60, 10, 60), "{:?}", run.drifts);
        assert_eq!(run.seeks, 0);
    }

    #[test]
    fn a_follower_ahead_converges_without_a_seek() {
        let run = simulate(1.0, 1.0, 0.45, FRAME_60, 72);

        assert!(run.stays_within(FRAME_60, 12, 60), "{:?}", run.drifts);
        assert_eq!(run.seeks, 0);
    }

    #[test]
    fn a_slow_master_is_learned_by_the_integral_trim() {
        let run = simulate(0.985, 1.0, 0.0, FRAME_60, 80);

        assert!(run.stays_within(FRAME_60, 20, 60), "{:?}", run.drifts);
        assert_eq!(run.seeks, 0);
    }

    #[test]
    fn a_slow_follower_is_learned_by_the_integral_trim() {
        let run = simulate(1.0, 0.97, -0.2, FRAME_60, 100);

        assert!(run.stays_within(FRAME_60, 40, 60), "{:?}", run.drifts);
        assert_eq!(run.seeks, 0);
    }

    #[test]
    fn a_slow_master_and_an_offset_together_converge_and_stay() {
        let run = simulate(0.985, 1.0, -0.3, FRAME_60, 120);

        assert!(run.stays_within(FRAME_60, 25, 60), "{:?}", run.drifts);
        assert!(run.worst_from(25) <= FRAME_60, "{:?}", run.drifts);
        assert_eq!(run.seeks, 0);
    }

    #[test]
    fn a_30_fps_video_is_held_to_its_own_frame() {
        let frame = frame_period(Some(30.0));

        let run = simulate(0.985, 1.0, 0.2, frame, 120);

        assert!(run.stays_within(frame, 25, 60), "{:?}", run.drifts);
    }

    #[test]
    fn a_drift_the_old_band_ignored_is_corrected_at_once() {
        // 25 ms is below the old fixed band of 1/30 s but above one frame at 60 fps.
        let mut follower = Follower::new();
        let first = follower.step(&tick(100.0, SECOND), SECOND, 100.025, None, FRAME_60);
        let run = simulate(1.0, 1.0, 0.025, FRAME_60, 8);

        assert!(speed_of(first) < 1.0, "{first:?}");
        assert!(run.drifts[5].abs() <= FRAME_60 / 4.0, "{:?}", run.drifts);
    }

    #[test]
    fn the_speed_stays_within_five_percent_of_the_masters() {
        for (master_rate, follower_rate, start) in [(1.0, 1.0, -0.4), (0.985, 1.0, 0.3), (1.0, 0.95, -0.4)] {
            let run = simulate(master_rate, follower_rate, start, FRAME_60, 60);

            assert!(
                run.speeds
                    .iter()
                    .all(|speed| (0.95 - 1e-12..=1.05 + 1e-12).contains(speed)),
                "{:?}",
                run.speeds
            );
        }
    }

    #[test]
    fn a_seek_clears_the_integral() {
        let mut follower = Follower::new();
        for k in 0..10_u64 {
            let now = (k + 1) * SECOND;
            follower.step(&tick(100.0, now), now, 100.05, None, FRAME_60);
        }
        assert!(follower.integral.abs() > 0.1, "{}", follower.integral);

        let action = follower.step(&tick(100.0, 20 * SECOND), 20 * SECOND, 101.0, None, FRAME_60);

        assert!(matches!(action, SyncAction::Seek(_)), "{action:?}");
        assert!(follower.integral.abs() < f64::EPSILON);
    }

    #[test]
    fn a_summary_reports_the_ticks_since_the_last_one_and_resets() {
        let mut follower = Follower::new();
        let drifts = [0.005, -0.005, 0.01, 0.002, -0.001];
        for (k, drift) in (1_u64..).zip(drifts) {
            follower.step(&tick(100.0, k * SECOND), k * SECOND, 100.0 + drift, None, FRAME_60);
        }

        let summary = follower.take_summary().expect("five ticks were taken");

        assert_eq!(summary.ticks, 5);
        assert!((summary.mean_abs_s - 0.0046).abs() < 1e-9, "{summary:?}");
        assert!((summary.max_abs_s - 0.01).abs() < 1e-9, "{summary:?}");
        assert_eq!(summary.speed_changes, 0);
        assert_eq!(summary.seeks, 0);
        assert_eq!(follower.take_summary(), None);
    }

    #[test]
    fn a_summary_counts_speed_changes_and_seeks() {
        let mut follower = Follower::new();

        follower.step(&tick(100.0, SECOND), SECOND, 99.8, None, FRAME_60);
        follower.step(&tick(100.0, 2 * SECOND), 2 * SECOND, 99.2, None, FRAME_60);
        let summary = follower.take_summary().expect("two ticks were taken");

        assert_eq!(summary.ticks, 2);
        assert_eq!(summary.speed_changes, 1);
        assert_eq!(summary.seeks, 1);
        assert!((summary.max_abs_s - 0.8).abs() < 1e-9, "{summary:?}");
        assert!((summary.mean_abs_s - 0.5).abs() < 1e-9, "{summary:?}");
    }

    #[test]
    fn a_follower_that_has_seen_no_tick_has_no_summary() {
        assert_eq!(Follower::new().take_summary(), None);
    }
}
