//! The follower's drift correction (D-02): given the master's clock and the follower's own position, decide whether
//! to leave playback alone, nudge its speed or seek. Pure, so the cases are tested without a player.
//!
//! The master publishes a [ClockTick]; a follower extrapolates it to "now" with the shared system clock, compares
//! that with its own position and answers with a [SyncAction].

use sucrose_ipc::protocol::ClockTick;

/// Drift up to this many seconds is left alone by default: one frame at 30 fps.
pub const DEFAULT_FRAME_S: f64 = 1.0 / 30.0;

/// Drift up to this many seconds is corrected by a speed change, beyond it by a seek.
pub const SEEK_THRESHOLD_S: f64 = 0.5;

/// The speed factor of a nudge: 2 % faster when behind, 2 % slower when ahead.
pub const SPEED_STEP: f64 = 0.02;

/// What a follower does about its drift.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncAction {
    /// Leave playback alone.
    None,
    /// Play at this speed (the master's speed times 0.98 or 1.02) until the drift is gone.
    Speed(f64),
    /// Jump to this position in seconds, exactly.
    Seek(f64),
}

/// Decides what a follower at `local_pos_s` does about the master's `tick`, read at `now_ns` on the same clock as
/// `tick.stamp_ns`. `duration_s` is the file length when it loops, so a wrap is not mistaken for drift; `frame_s` is
/// the drift that is left alone. A paused master gives nothing to chase.
#[must_use]
pub fn correct(tick: &ClockTick, now_ns: u64, local_pos_s: f64, duration_s: Option<f64>, frame_s: f64) -> SyncAction {
    // The loop end is not handled yet: a follower across a wrap looks far off and is seeked.
    let _ = duration_s;
    if tick.paused {
        return SyncAction::None;
    }
    let expected = expected_position(tick, now_ns);
    let drift = local_pos_s - expected;
    if drift.abs() <= frame_s {
        SyncAction::None
    } else if drift.abs() <= SEEK_THRESHOLD_S {
        // Ahead of the master: slow down. Behind it: speed up.
        let factor = if drift > 0.0 {
            1.0 - SPEED_STEP
        } else {
            1.0 + SPEED_STEP
        };
        SyncAction::Speed(tick.speed * factor)
    } else {
        SyncAction::Seek(expected)
    }
}

/// Where the master is at `now_ns`: its position when the tick was taken plus the time since, at its speed.
fn expected_position(tick: &ClockTick, now_ns: u64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let elapsed_s = now_ns.saturating_sub(tick.stamp_ns) as f64 / 1e9;
    tick.position_s + elapsed_s * tick.speed
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: u64 = 1_000_000_000;

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

    #[test]
    fn drift_inside_one_frame_is_left_alone() {
        let action = correct(&tick(10.0, 5 * SECOND), 5 * SECOND, 10.01, None, DEFAULT_FRAME_S);

        assert_eq!(action, SyncAction::None);
    }

    #[test]
    fn a_follower_ahead_by_a_fifth_of_a_second_slows_down() {
        let action = correct(&tick(10.0, 5 * SECOND), 5 * SECOND, 10.2, None, DEFAULT_FRAME_S);

        assert!((speed_of(action) - 0.98).abs() < 1e-9);
    }

    #[test]
    fn a_follower_behind_by_a_fifth_of_a_second_speeds_up() {
        let action = correct(&tick(10.0, 5 * SECOND), 5 * SECOND, 9.8, None, DEFAULT_FRAME_S);

        assert!((speed_of(action) - 1.02).abs() < 1e-9);
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
}
