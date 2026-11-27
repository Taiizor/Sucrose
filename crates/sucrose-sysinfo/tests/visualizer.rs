//! The audio visualizer through its public API: how the envelope behaves over a sequence of results, as the legacy
//! `AudioVisualizer.cs` does (attack 0.35, release 0.06), and that a signal moves across the bands with its frequency.

use sucrose_sysinfo::visualizer::{BAND_COUNT, Bands, Visualizer};

const RATE: u32 = 48_000;
/// 16 ms of audio at 48 kHz: the audio that has to be pushed between two results.
const INTERVAL: usize = 768;
/// The release coefficient of the legacy envelope.
const RELEASE: f64 = 0.06;

/// Stereo frames of a sine at `hz` with `amplitude`, `frames` long, starting at frame `start`.
fn sine(hz: f64, amplitude: f64, start: usize, frames: usize) -> Vec<f32> {
    (0..frames)
        .flat_map(|n| {
            let t = (start + n) as f64 / f64::from(RATE);
            let v = (amplitude * (2.0 * std::f64::consts::PI * hz * t).sin()) as f32;
            [v, v]
        })
        .collect()
}

fn argmax(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| index)
        .expect("128 values")
}

/// A visualizer that has heard a loud 1 kHz tone and then 4096 frames of silence, so its ring holds only silence and
/// every further result is the release envelope alone.
fn after_a_loud_signal() -> (Visualizer, Bands) {
    let mut visualizer = Visualizer::new(RATE);
    visualizer.push(&sine(1000.0, 0.8, 0, 8192), 2);
    for _ in 0..5 {
        visualizer.push(&sine(1000.0, 0.8, 8192, INTERVAL), 2);
        visualizer.take();
    }
    visualizer.push(&vec![0.0; 4096 * 2], 2);
    let last = visualizer.take().expect("a result after the silence");
    (visualizer, last)
}

#[test]
fn silence_after_a_loud_signal_lowers_every_band_by_the_release_factor_per_result_until_it_reaches_zero() {
    let (mut visualizer, mut previous) = after_a_loud_signal();
    assert!(
        previous.mono.iter().any(|v| *v > 0.05),
        "the loud signal left an envelope"
    );

    let mut results = 0;
    while !visualizer.is_idle() {
        visualizer.push(&vec![0.0; INTERVAL * 2], 2);
        let next = visualizer.take().expect("16 ms of audio was pushed");
        for (name, before, after) in [
            ("mono", &previous.mono, &next.mono),
            ("left", &previous.left, &next.left),
            ("right", &previous.right, &next.right),
        ] {
            for (band, (b, a)) in before.iter().zip(after).enumerate() {
                // The neighbour average is linear, so the output falls by the same factor as the envelope, give or take the
                // step that snaps an envelope value below 0.0001 to 0 (at most that much, spread over three bands).
                if *b > 1e-3 {
                    assert!(
                        (a - b * (1.0 - RELEASE)).abs() < 1e-4,
                        "{name} band {band}: {b} -> {a}, expected {}",
                        b * (1.0 - RELEASE)
                    );
                }
                assert!(a <= b, "{name} band {band} never rises on silence");
            }
        }
        previous = next;
        results += 1;
        assert!(results < 400, "the bands must reach 0");
    }
    let last = visualizer.take();
    assert!(last.is_none() || last == Some(Bands::silent()));
    assert!(results > 30, "the release is gradual, not a cut: {results}");
}

#[test]
fn a_signal_attacks_faster_than_it_releases() {
    let mut visualizer = Visualizer::new(RATE);
    visualizer.push(&sine(1000.0, 0.8, 0, 4096), 2);
    let first = visualizer.take().expect("the ring is full");
    visualizer.push(&sine(1000.0, 0.8, 4096, INTERVAL), 2);
    let second = visualizer.take().expect("16 ms later");

    let peak = argmax(&first.mono);
    assert!(
        second.mono[peak] > first.mono[peak],
        "the envelope keeps rising toward the level"
    );
    assert!(
        first.mono[peak] < 0.5,
        "one attack step moves a bar by 35 percent of the way, not all of it: {}",
        first.mono[peak]
    );
}

#[test]
fn a_chirp_moves_the_peak_across_the_bands() {
    let mut visualizer = Visualizer::new(RATE);
    let mut peaks = Vec::new();
    let mut start = 0;
    // A sweep from 100 Hz to 10 kHz over about 1.5 seconds, pushed in 16 ms steps.
    for step in 0..90 {
        let hz = 100.0 * (100.0_f64).powf(f64::from(step) / 89.0);
        visualizer.push(&sine(hz, 0.5, start, INTERVAL), 2);
        start += INTERVAL;
        if let Some(bands) = visualizer.take() {
            assert!(bands.mono.iter().all(|v| (0.0..=1.0).contains(v)));
            peaks.push(argmax(&bands.mono));
        }
    }

    let first = peaks.first().copied().expect("results");
    let last = peaks.last().copied().expect("results");
    assert!(first < 60, "the low end of the sweep sits in the low bands: {first}");
    assert!(last > 90, "the high end of the sweep sits in the high bands: {last}");
    assert!(
        peaks.windows(2).all(|pair| pair[1] + 8 >= pair[0]),
        "the peak does not run backwards: {peaks:?}"
    );
    assert!(last < BAND_COUNT);
}

#[test]
fn tones_of_rising_pitch_peak_in_rising_bands() {
    let bands_of = |hz: f64| {
        let mut visualizer = Visualizer::new(RATE);
        visualizer.push(&sine(hz, 0.5, 0, 4096), 2);
        argmax(&visualizer.take().expect("the ring is full").mono)
    };

    let found: Vec<usize> = [60.0, 250.0, 1000.0, 4000.0, 12_000.0]
        .iter()
        .map(|hz| bands_of(*hz))
        .collect();

    assert!(found.windows(2).all(|pair| pair[0] < pair[1]), "{found:?}");
}
