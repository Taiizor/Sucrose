//! The audio visualizer of the legacy app (`AudioVisualizer.cs`, docs/legacy/services.md section 1.2) as a pure module: interleaved
//! samples go in, 128 log-spaced band magnitudes per channel come out. Nothing here touches the OS, so the algorithm is
//! tested without an audio device; the loopback capture that feeds it is [`crate::audio`].
//!
//! The steps follow the legacy code: a ring of 4096 samples per channel, a Hann window, an FFT without scaling, 128 bands
//! spread logarithmically from 20 Hz to the smaller of 16 kHz and the Nyquist frequency with the peak magnitude of each
//! band, a +2 dB/octave tilt, the magnitude scaled by 2.5 over the magnitude of a full-scale tone minus a 0.01 floor and
//! clamped to 0..1, an attack/release envelope per band and a neighbour average. The mono spectrum is the magnitude of
//! (L + R) / 2. A result comes at most once per 16 ms of audio.

use std::sync::Arc;

use realfft::{RealFftPlanner, RealToComplex, num_complex::Complex};

/// How many bands every channel has (`MaxSample` in the legacy code): the length of `Data`, `DataLeft` and `DataRight`.
pub const BAND_COUNT: usize = 128;

/// The fixed FFT size, a power of two independent of the size of the packets the device delivers.
const FFT_SIZE: usize = 4096;
/// The bins above DC up to Nyquist.
const HALF_FFT: usize = FFT_SIZE / 2;
/// The frequency range the bands cover, on a logarithmic scale.
const MIN_FREQUENCY: f64 = 20.0;
const MAX_FREQUENCY: f64 = 16_000.0;
/// Magnitude over full scale is multiplied by this before the clamp.
const LINEAR_GAIN: f64 = 2.5;
/// Subtracted before the clamp so residual hiss stays flat.
const NOISE_FLOOR: f64 = 0.01;
/// The high-frequency tilt in dB per octave, anchored at `MIN_FREQUENCY`.
const TILT_DB_PER_OCTAVE: f64 = 2.0;
/// The envelope coefficients per result: fast rise, slow fall.
const ATTACK_SPEED: f64 = 0.35;
const RELEASE_SPEED: f64 = 0.06;
/// How many neighbours on each side the output average takes in.
const HORIZONTAL_SMOOTHNESS: usize = 1;
/// The most frequent results come, in milliseconds of audio.
const EMIT_INTERVAL_MS: usize = 16;
/// An envelope value below this while the input is silent is set to 0, so a page sees silence as 0 and the capture can
/// tell the visualizer has settled (the legacy envelope only approaches 0). The step is far below what a bar can show.
const SETTLE_BELOW: f64 = 1e-4;

/// One result: 128 values in [0, 1] for each channel and for the mono mix.
#[derive(Debug, Clone, PartialEq)]
pub struct Bands {
    /// The left channel.
    pub left: Vec<f64>,
    /// The right channel.
    pub right: Vec<f64>,
    /// The mix of both, `Data` in the legacy payload.
    pub mono: Vec<f64>,
}

impl Bands {
    /// 128 zeros in every channel: a device that plays nothing.
    #[must_use]
    pub fn silent() -> Self {
        Self {
            left: vec![0.0; BAND_COUNT],
            right: vec![0.0; BAND_COUNT],
            mono: vec![0.0; BAND_COUNT],
        }
    }
}

/// The raw peak magnitude of each band, before scaling.
#[derive(Debug, Clone, PartialEq)]
struct Peaks {
    mono: Vec<f64>,
    left: Vec<f64>,
    right: Vec<f64>,
}

/// The visualizer of one capture format: it keeps the sample rings and the envelopes between pushes.
pub struct Visualizer {
    sample_rate: u32,
    band_start: Vec<usize>,
    band_end: Vec<usize>,
    tilt_gain: Vec<f64>,
    window: Vec<f64>,
    /// The bin magnitude of a full-scale tone through the window: the 0 dBFS anchor.
    reference: f64,
    left_ring: Vec<f32>,
    right_ring: Vec<f32>,
    write_pos: usize,
    filled: bool,
    frames_since_emit: usize,
    emit_interval: usize,
    fft: Arc<dyn RealToComplex<f64>>,
    input_left: Vec<f64>,
    input_right: Vec<f64>,
    output_left: Vec<Complex<f64>>,
    output_right: Vec<Complex<f64>>,
    scratch: Vec<Complex<f64>>,
    envelope_mono: Vec<f64>,
    envelope_left: Vec<f64>,
    envelope_right: Vec<f64>,
}

impl Visualizer {
    /// A visualizer for audio at `sample_rate` Hz.
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        let sample_rate = sample_rate.max(1);
        let fft = RealFftPlanner::<f64>::new().plan_fft_forward(FFT_SIZE);
        let scratch = fft.make_scratch_vec();
        let window: Vec<f64> = (0..FFT_SIZE)
            .map(|n| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * n as f64 / (FFT_SIZE - 1) as f64).cos()))
            .collect();
        let reference = window.iter().sum::<f64>() / 2.0;
        let (band_start, band_end, tilt_gain) = build_bands(sample_rate);
        Self {
            sample_rate,
            band_start,
            band_end,
            tilt_gain,
            window,
            reference,
            left_ring: vec![0.0; FFT_SIZE],
            right_ring: vec![0.0; FFT_SIZE],
            write_pos: 0,
            filled: false,
            // The first result comes as soon as the ring is full.
            frames_since_emit: usize::MAX / 2,
            emit_interval: (sample_rate as usize * EMIT_INTERVAL_MS / 1000).max(1),
            input_left: fft.make_input_vec(),
            input_right: fft.make_input_vec(),
            output_left: fft.make_output_vec(),
            output_right: fft.make_output_vec(),
            fft,
            scratch,
            envelope_mono: vec![0.0; BAND_COUNT],
            envelope_left: vec![0.0; BAND_COUNT],
            envelope_right: vec![0.0; BAND_COUNT],
        }
    }

    /// The sample rate this visualizer was built for.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Adds interleaved samples of `channels` channels: the first is the left channel, the second the right, a
    /// mono device feeds both with its only channel, further channels are ignored.
    pub fn push(&mut self, interleaved: &[f32], channels: usize) {
        let channels = channels.max(1);
        for frame in interleaved.chunks_exact(channels) {
            let left = frame[0];
            let right = if channels >= 2 { frame[1] } else { left };
            self.left_ring[self.write_pos] = left;
            self.right_ring[self.write_pos] = right;
            self.write_pos += 1;
            if self.write_pos >= FFT_SIZE {
                self.write_pos = 0;
                self.filled = true;
            }
            self.frames_since_emit = self.frames_since_emit.saturating_add(1);
        }
    }

    /// The next result, or `None` while the ring is still filling or less than 16 ms of audio has been pushed since
    /// the previous result.
    pub fn take(&mut self) -> Option<Bands> {
        if !self.filled || self.frames_since_emit < self.emit_interval {
            return None;
        }
        self.frames_since_emit = 0;
        let peaks = self.peaks();
        Some(Bands {
            mono: finalize(&peaks.mono, &mut self.envelope_mono, &self.tilt_gain, self.reference),
            left: finalize(&peaks.left, &mut self.envelope_left, &self.tilt_gain, self.reference),
            right: finalize(&peaks.right, &mut self.envelope_right, &self.tilt_gain, self.reference),
        })
    }

    /// Whether every envelope is at 0: feeding silence changes nothing any more.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        [&self.envelope_mono, &self.envelope_left, &self.envelope_right]
            .iter()
            .all(|envelope| envelope.iter().all(|value| *value == 0.0))
    }

    /// Windows the latest 4096 samples of each channel (the oldest first), transforms them and takes the peak magnitude of
    /// every band. The mono magnitude is that of (L + R) / 2, which is exact because the transform is linear.
    fn peaks(&mut self) -> Peaks {
        let mut index = self.write_pos;
        for n in 0..FFT_SIZE {
            self.input_left[n] = f64::from(self.left_ring[index]) * self.window[n];
            self.input_right[n] = f64::from(self.right_ring[index]) * self.window[n];
            index += 1;
            if index >= FFT_SIZE {
                index = 0;
            }
        }
        // The buffers have the lengths the plan made them with, so the transform cannot fail; if it ever did, the bands
        // would read as silence rather than stop the capture.
        let left_ok = self
            .fft
            .process_with_scratch(&mut self.input_left, &mut self.output_left, &mut self.scratch)
            .is_ok();
        let right_ok = self
            .fft
            .process_with_scratch(&mut self.input_right, &mut self.output_right, &mut self.scratch)
            .is_ok();
        let mut peaks = Peaks {
            mono: vec![0.0; BAND_COUNT],
            left: vec![0.0; BAND_COUNT],
            right: vec![0.0; BAND_COUNT],
        };
        if !(left_ok && right_ok) {
            return peaks;
        }
        for band in 0..BAND_COUNT {
            let (mut mono, mut left, mut right) = (0.0_f64, 0.0_f64, 0.0_f64);
            for bin in self.band_start[band]..self.band_end[band] {
                let l = self.output_left[bin];
                let r = self.output_right[bin];
                mono = mono.max(((l + r) * 0.5).norm());
                left = left.max(l.norm());
                right = right.max(r.norm());
            }
            peaks.mono[band] = mono;
            peaks.left[band] = left;
            peaks.right[band] = right;
        }
        peaks
    }
}

/// The first bin, the end bin (exclusive) and the tilt gain of each band for audio at `rate` Hz.
fn build_bands(rate: u32) -> (Vec<usize>, Vec<usize>, Vec<f64>) {
    let bin_hz = f64::from(rate) / FFT_SIZE as f64;
    let log_min = MIN_FREQUENCY.ln();
    let log_max = MAX_FREQUENCY.min(f64::from(rate) / 2.0).ln();
    let count = BAND_COUNT as f64;
    let mut starts = Vec::with_capacity(BAND_COUNT);
    let mut ends = Vec::with_capacity(BAND_COUNT);
    let mut tilts = Vec::with_capacity(BAND_COUNT);
    for i in 0..BAND_COUNT {
        let f0 = (log_min + (log_max - log_min) * i as f64 / count).exp();
        let f1 = (log_min + (log_max - log_min) * (i + 1) as f64 / count).exp();
        let mut start = (f0 / bin_hz).floor() as usize;
        let mut end = (f1 / bin_hz).floor() as usize;
        if start < 1 {
            // Skip DC.
            start = 1;
        }
        if end <= start {
            // At least one bin per band.
            end = start + 1;
        }
        end = end.min(HALF_FFT);
        starts.push(start);
        ends.push(end);
        let center = (log_min + (log_max - log_min) * (i as f64 + 0.5) / count).exp();
        tilts.push((center / MIN_FREQUENCY).powf(TILT_DB_PER_OCTAVE / (20.0 * 2.0_f64.log10())));
    }
    (starts, ends, tilts)
}

/// Scales the peak magnitudes to [0, 1], moves the envelope toward them (fast up, slow down) and averages each band with
/// its neighbours into a fresh array.
fn finalize(peaks: &[f64], envelope: &mut [f64], tilt: &[f64], reference: f64) -> Vec<f64> {
    for ((peak, value), tilt) in peaks.iter().zip(envelope.iter_mut()).zip(tilt) {
        let normalized = if *peak <= 0.0 {
            0.0
        } else {
            ((peak * tilt / reference * LINEAR_GAIN) - NOISE_FLOOR).clamp(0.0, 1.0)
        };
        let coefficient = if normalized > *value {
            ATTACK_SPEED
        } else {
            RELEASE_SPEED
        };
        *value += (normalized - *value) * coefficient;
        if normalized < SETTLE_BELOW && *value < SETTLE_BELOW {
            *value = 0.0;
        }
    }
    smooth(envelope)
}

/// Each value averaged with its neighbours up to `HORIZONTAL_SMOOTHNESS` away; the edges average over what exists.
fn smooth(envelope: &[f64]) -> Vec<f64> {
    (0..envelope.len())
        .map(|i| {
            let from = i.saturating_sub(HORIZONTAL_SMOOTHNESS);
            let to = (i + HORIZONTAL_SMOOTHNESS).min(envelope.len() - 1);
            envelope[from..=to].iter().sum::<f64>() / (to - from + 1) as f64
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 48_000;

    /// `frames` stereo frames of a sine at `hz` with `amplitude`, the same in both channels.
    fn sine(hz: f64, amplitude: f64, frames: usize, start: usize) -> Vec<f32> {
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
            .map(|(i, _)| i)
            .expect("128 values")
    }

    /// The band whose range holds `hz`, from the band formula (independent of the bin arithmetic).
    fn band_of(hz: f64) -> usize {
        let (log_min, log_max) = (MIN_FREQUENCY.ln(), MAX_FREQUENCY.min(f64::from(RATE) / 2.0).ln());
        let position = (hz.ln() - log_min) / (log_max - log_min) * BAND_COUNT as f64;
        position.floor() as usize
    }

    #[test]
    fn a_one_kilohertz_sine_peaks_in_the_band_that_holds_one_kilohertz() {
        let mut visualizer = Visualizer::new(RATE);
        visualizer.push(&sine(1000.0, 0.5, RATE as usize / 5, 0), 2);

        let bands = visualizer.take().expect("the ring is full");

        let expected = band_of(1000.0);
        for (name, values) in [("left", &bands.left), ("right", &bands.right), ("mono", &bands.mono)] {
            assert_eq!(values.len(), BAND_COUNT, "{name}");
            assert!(values.iter().all(|v| (0.0..=1.0).contains(v)), "{name} stays in 0..=1");
            // The neighbour average can move the top of a peak that sits at a band edge by one band.
            assert!(
                argmax(values).abs_diff(expected) <= 1,
                "{name}: {} vs {expected}",
                argmax(values)
            );
            assert!(values[argmax(values)] > 0.05, "{name} has a visible peak");
        }
    }

    #[test]
    fn a_tone_in_the_middle_of_a_band_peaks_in_exactly_that_band() {
        let (log_min, log_max) = (MIN_FREQUENCY.ln(), MAX_FREQUENCY.min(f64::from(RATE) / 2.0).ln());
        let hz = (log_min + (log_max - log_min) * 64.5 / BAND_COUNT as f64).exp();
        let mut visualizer = Visualizer::new(RATE);
        visualizer.push(&sine(hz, 0.5, RATE as usize / 5, 0), 2);

        let bands = visualizer.take().expect("the ring is full");

        assert_eq!(argmax(&bands.mono), 64);
    }

    #[test]
    fn silence_gives_all_zero_bands() {
        let mut visualizer = Visualizer::new(RATE);
        visualizer.push(&vec![0.0; FFT_SIZE * 2], 2);

        let bands = visualizer.take().expect("the ring is full");

        assert_eq!(bands, Bands::silent());
        assert!(visualizer.is_idle());
    }

    #[test]
    fn a_left_only_signal_leaves_the_right_bands_at_zero_and_the_mono_peaks_at_half() {
        let mut visualizer = Visualizer::new(RATE);
        let stereo: Vec<f32> = sine(1000.0, 0.5, FFT_SIZE, 0)
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|frame| [frame[0], 0.0])
            .collect();
        visualizer.push(&stereo, 2);

        let peaks = visualizer.peaks();

        assert!(
            peaks.right.iter().all(|v| *v == 0.0),
            "no right channel, no right peaks"
        );
        assert!(peaks.left[band_of(1000.0)] > 0.0);
        for (mono, left) in peaks.mono.iter().zip(&peaks.left) {
            assert!(
                (mono - left * 0.5).abs() <= 1e-9 * left.max(1.0),
                "{mono} vs half of {left}"
            );
        }
    }

    #[test]
    fn a_mono_device_feeds_both_channels() {
        let mut visualizer = Visualizer::new(RATE);
        let mono: Vec<f32> = sine(1000.0, 0.5, FFT_SIZE, 0)
            .as_chunks::<2>()
            .0
            .iter()
            .map(|frame| frame[0])
            .collect();
        visualizer.push(&mono, 1);

        let bands = visualizer.take().expect("the ring is full");

        assert_eq!(bands.left, bands.right);
        assert!(bands.left[band_of(1000.0)] > 0.0);
    }

    #[test]
    fn a_result_comes_at_most_once_per_sixteen_milliseconds_of_audio() {
        let mut visualizer = Visualizer::new(RATE);
        let interval = RATE as usize * EMIT_INTERVAL_MS / 1000;

        visualizer.push(&vec![0.0; (FFT_SIZE - 1) * 2], 2);
        assert!(visualizer.take().is_none(), "the ring is not full yet");
        visualizer.push(&[0.0, 0.0], 2);
        assert!(
            visualizer.take().is_some(),
            "the first result comes when the ring fills"
        );
        assert!(visualizer.take().is_none(), "nothing new was pushed");
        visualizer.push(&vec![0.0; (interval - 1) * 2], 2);
        assert!(visualizer.take().is_none(), "just under 16 ms");
        visualizer.push(&[0.0, 0.0], 2);
        assert!(visualizer.take().is_some(), "16 ms of audio");
    }

    #[test]
    fn the_neighbour_average_covers_what_exists_at_the_first_and_last_band() {
        let mut envelope = vec![0.0; BAND_COUNT];
        envelope[0] = 1.0;
        envelope[BAND_COUNT - 1] = 1.0;

        let smoothed = smooth(&envelope);

        assert!((smoothed[0] - 0.5).abs() < 1e-12, "(1 + 0) / 2");
        assert!((smoothed[1] - 1.0 / 3.0).abs() < 1e-12);
        assert!((smoothed[BAND_COUNT - 2] - 1.0 / 3.0).abs() < 1e-12);
        assert!((smoothed[BAND_COUNT - 1] - 0.5).abs() < 1e-12);
        assert_eq!(smoothed[64], 0.0);
    }

    #[test]
    fn the_bands_follow_the_legacy_edges() {
        let (starts, ends, tilts) = build_bands(RATE);

        assert_eq!(starts.len(), BAND_COUNT);
        assert_eq!(starts[0], 1, "DC is skipped");
        assert!(
            starts.iter().zip(&ends).all(|(s, e)| s < e),
            "at least one bin per band"
        );
        assert!(ends.iter().all(|e| *e <= HALF_FFT));
        assert!(
            tilts.windows(2).all(|pair| pair[0] < pair[1]),
            "the tilt rises with frequency"
        );
        // 16 kHz is about 1365 bins at 48 kHz.
        assert!(
            (1360..=1370).contains(&ends[BAND_COUNT - 1]),
            "{}",
            ends[BAND_COUNT - 1]
        );
        assert!((tilts[0] - 1.0).abs() < 0.01, "the tilt is anchored at 20 Hz");
    }

    #[test]
    fn a_low_sample_rate_ends_the_bands_at_nyquist() {
        let (_, ends, _) = build_bands(8000);

        assert!(ends.iter().all(|e| *e <= HALF_FFT));
        assert!(
            ends[BAND_COUNT - 1] >= 2000,
            "the last band reaches 4 kHz, the Nyquist frequency"
        );
    }
}
