/*
MIT License

Copyright (c) 2026 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
//! High performance `no_std` lowpass filter for digital signal processing.
//!
//! This crate implements a simple first-order digital lowpass filter for
//! `f32` and `f64` samples. Use it, for example, to extract the bass from a
//! song or to smooth noisy sensor data. It has no dependencies, no `unsafe`
//! code, and performs no allocations, making it suitable for any target from
//! desktop to embedded.
//!
//! Samples should be in range `-1.0..=1.0`, which is the default in DSP.
//!
//! ## Usage
//!
//! Create a [`LowpassFilter`] and pass your samples to
//! [`LowpassFilter::run_slice`]. It processes samples in blocks that
//! compilers auto-vectorize (SIMD), several times faster than per-sample
//! processing.
//!
//! ```rust
//! use lowpass_filter::LowpassFilter;
//!
//! // Mono audio samples, recorded at 44.1 kHz sample rate.
//! let mut samples = [0.0_f32, 0.3, -0.6, 0.8, 0.5, -0.2];
//! // Only keep frequencies below 120 Hz; mutates the buffer in-place.
//! LowpassFilter::new(44100.0, 120.0).run_slice(&mut samples);
//! ```
//!
//! For streaming data, e.g. in an audio callback, create the filter once and
//! reuse it. Its state carries over between calls, so filtering chunk by
//! chunk equals filtering everything at once. A new filter per chunk starts
//! from silence at every chunk boundary, which distorts the signal.
//!
//! ```rust
//! use lowpass_filter::LowpassFilter;
//!
//! let mut filter = LowpassFilter::<f32>::new(44100.0, 120.0);
//! for mut chunk in [[0.0, 0.3, -0.6, 0.8], [0.5, -0.2, 0.1, 0.4]] {
//!     filter.run_slice(&mut chunk);
//! }
//! ```
//!
//! For samples that do not live in a slice, filter them one at a time with
//! [`LowpassFilter::run`]. See [`LowpassFilter`] for more examples.

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::must_use_candidate,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]
#![no_std]

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

use core::fmt::{Debug, Display};
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

mod sealed {
    /// Seals [`super::Sample`] so it cannot be implemented outside this
    /// crate.
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// A sample type [`LowpassFilter`] can operate on: [`f32`] or [`f64`].
///
/// This trait is sealed and cannot be implemented outside of this crate.
pub trait Sample:
    sealed::Sealed
    + Copy
    + PartialOrd
    + Debug
    + Display
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    /// `0.0`
    const ZERO: Self;
    /// `1.0`
    const ONE: Self;
    /// `2.0`
    const TWO: Self;
    /// Archimedes' constant (π).
    const PI: Self;
}

impl Sample for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const PI: Self = core::f32::consts::PI;
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const PI: Self = core::f64::consts::PI;
}

/// Block size for slice processing. 8 measured fastest on x86-64 for f32
/// and f64.
const LANES: usize = 8;

/// A first-order lowpass filter for `f32` and `f64` samples.
///
/// The filter is stateful: each output depends on the previous one. Filter
/// samples in-place with [`Self::run_slice`] (preferred, significantly
/// faster) or one at a time with [`Self::run`]. Both continue from the same
/// state and can be mixed freely. Use [`Self::reset`] before filtering an
/// unrelated signal.
///
/// It is recommended to operate on values in range `-1.0..=1.0`, which is also
/// the default in DSP. All values must be finite, i.e., not NaN or infinite.
///
/// Using `f32` (the default) is recommended: for audio processing, it is
/// accurate enough and significantly faster than `f64`.
///
/// # Examples
///
/// Filter a whole buffer in one go:
///
/// ```rust
/// use lowpass_filter::LowpassFilter;
///
/// let mut samples = [0.0_f32, 0.3, -0.6, 0.8, 0.5, -0.2];
/// LowpassFilter::new(44100.0, 120.0).run_slice(&mut samples);
/// ```
///
/// Streaming: keep one filter per signal for its whole lifetime, e.g. in
/// your audio processor. Its state carries over between calls, so filtering
/// buffer by buffer equals filtering everything at once:
///
/// ```rust
/// use lowpass_filter::LowpassFilter;
///
/// struct BassExtractor {
///     filter: LowpassFilter,
/// }
///
/// impl BassExtractor {
///     /// Called by the audio backend for each new buffer.
///     fn process(&mut self, buffer: &mut [f32]) {
///         self.filter.run_slice(buffer);
///     }
/// }
///
/// let mut bass = BassExtractor {
///     filter: LowpassFilter::new(44100.0, 120.0),
/// };
/// bass.process(&mut [0.0, 0.3, -0.6, 0.8]);
/// bass.process(&mut [0.5, -0.2, 0.1, 0.4]);
/// ```
///
/// Interleaved stereo: every channel is a separate signal and needs its own
/// filter state. For large buffers, deinterleaving into one buffer per
/// channel and using [`Self::run_slice`] might be faster.
///
/// ```rust
/// use lowpass_filter::LowpassFilter;
///
/// // [l, r, l, r, ...]
/// let mut stereo = [0.1_f32, -0.1, 0.4, -0.4, 0.2, -0.2];
/// let mut left = LowpassFilter::new(44100.0, 120.0);
/// let mut right = left.clone();
/// for frame in stereo.chunks_exact_mut(2) {
///     frame[0] = left.run(frame[0]);
///     frame[1] = right.run(frame[1]);
/// }
/// ```
///
/// Smoothing values as they arrive, e.g. sensor readings:
///
/// ```rust
/// use lowpass_filter::LowpassFilter;
///
/// // 100 Hz sensor, suppress jitter above 5 Hz
/// let mut filter = LowpassFilter::<f64>::new(100.0, 5.0);
/// let readings = [0.50, 0.52, 0.91, 0.49, 0.51];
/// let smoothed = readings.map(|reading| filter.run(reading));
/// ```
///
/// # More Info
/// - <https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter>
#[derive(Debug, Clone)]
pub struct LowpassFilter<T = f32> {
    alpha: T,
    /// Precomputed `1 - alpha`.
    beta: T,
    prev: T,
    carry_coeffs: [T; LANES],
    weights: [[T; LANES]; LANES],
}

impl<T: Sample> LowpassFilter<T> {
    /// Create a new lowpass filter.
    ///
    /// # Arguments
    /// - `sample_rate_hz`: Sample rate in Hz (e.g., 48000.0).
    /// - `cutoff_frequency_hz`: Cutoff frequency in Hz (e.g., 1000.0).
    ///
    /// # Panics
    /// Panics if `cutoff_frequency_hz` is above the Nyquist frequency, i.e.,
    /// half of `sample_rate_hz`.
    #[must_use]
    pub fn new(sample_rate_hz: T, cutoff_frequency_hz: T) -> Self {
        // Nyquist rule
        assert!(cutoff_frequency_hz * T::TWO <= sample_rate_hz);

        let rc = T::ONE / (cutoff_frequency_hz * T::TWO * T::PI);
        let dt = T::ONE / sample_rate_hz;
        let alpha = dt / (rc + dt);
        let beta = T::ONE - alpha;
        let (carry_coeffs, weights) = Self::precompute_slice_coefficients(alpha, beta);

        Self {
            alpha,
            beta,
            prev: T::ZERO,
            carry_coeffs,
            weights,
        }
    }

    /// Precomputes the coefficients for the slice processing.
    fn precompute_slice_coefficients(alpha: T, beta: T) -> ([T; LANES], [[T; LANES]; LANES]) {
        // Coefficients of the closed block form:
        //
        //   y[i] = beta^(i+1) * prev + sum(alpha * beta^(i-j) * x[j] for j <= i)
        // pow[k] = beta^k
        //
        // # Math
        // Unrolling `y[n] = alpha * x[n] + beta * y[n-1]` over a block of
        // samples yields
        //
        // y[i] = beta^(i+1) * prev + sum(alpha * beta^(i-j) * x[j] for j <= i)
        //
        // so within a block, samples only depend on the state `prev` from
        // before the block and can be computed in parallel, which enables
        // compiler auto-vectorization (SIMD). Only `prev` propagates serially
        // between blocks.
        let mut pow_coeffs = [T::ONE; LANES];
        for k in 1..LANES {
            pow_coeffs[k] = pow_coeffs[k - 1] * beta;
        }
        // carry_coeffs[i] = beta^(i+1), the weight of `prev` in y[i]
        let carry_coeffs = pow_coeffs.map(|p| p * beta);

        // cols[j][i] = alpha * beta^(i-j): the weight of input x[j] in output
        // y[i], stored as one "column" per input j so that the hot loop can
        // apply one sample to all outputs at once. Entries for i < j stay 0,
        // as later inputs cannot affect earlier outputs.
        //
        // In a nutshell: these are the constant factors of the filter formula
        // expanded over LANES samples. They only depend on alpha and beta, so
        // they can be computed once per filter instead of once per call.
        let mut weights = [[T::ZERO; LANES]; LANES];
        for (j, col) in weights.iter_mut().enumerate() {
            for (i, weight) in col.iter_mut().enumerate().skip(j) {
                *weight = alpha * pow_coeffs[i - j];
            }
        }

        (carry_coeffs, weights)
    }

    /// Filter a single sample and return the filtered result.
    ///
    /// For samples in a slice, prefer [`Self::run_slice`], which is
    /// significantly faster.
    ///
    /// It is recommended to operate on values in range `-1.0..=1.0`, which is
    /// also the default in DSP. All values must be finite, i.e., not NaN or
    /// infinite.
    #[inline]
    #[must_use]
    pub fn run(&mut self, input: T) -> T {
        // Re-associated form of `prev + alpha * (input - prev)`:
        //
        // On the very first iteration, the second part is zero and `input`
        // is only influenced by `alpha`.
        self.prev = self.alpha * input + self.beta * self.prev;
        self.prev
    }

    /// Filter a whole slice of samples in-place.
    ///
    /// Matches calling [`Self::run`] per sample up to tiny floating point
    /// rounding differences (roughly `1e-6` for `f32`), but is significantly
    /// faster. The filter state is updated, so consecutive calls compose, also
    /// when mixed with [`Self::run`].
    ///
    /// It is recommended to operate on values in range `-1.0..=1.0`, which is
    /// also the default in DSP. All values must be finite, i.e., not NaN or
    /// infinite.
    ///
    /// # Arguments
    /// - `samples`: Samples to filter in-place, preferably in range
    ///   `-1.0..=1.0`.
    #[inline]
    pub fn run_slice(&mut self, samples: &mut [T]) {
        if samples.is_empty() {
            return;
        }

        // Hot loop. `acc[i]` accumulates y[i] of the current block.
        // Its shape helps the compilers auto-vectorizer.
        let (chunks, remainder) = samples.as_chunks_mut::<LANES>();

        // Fast path for chunks using the precomputed coefficients.
        for chunk in chunks {
            let mut acc = [T::ZERO; LANES];
            // acc[i] = sum(weights[j][i] * x[j] for all j)
            for (col, &sample) in self.weights.iter().zip(chunk.iter()) {
                for (acc, &coeff) in acc.iter_mut().zip(col.iter()) {
                    *acc += coeff * sample;
                }
            }
            // acc[i] += beta^(i+1) * prev; the only place where state from
            // before the block enters.
            for (acc, &coeff) in acc.iter_mut().zip(self.carry_coeffs.iter()) {
                *acc += coeff * self.prev;
            }

            self.prev = acc[LANES - 1];
            for (sample, acc) in chunk.iter_mut().zip(acc.iter()) {
                *sample = *acc;
            }
        }

        // Process the tail (the leftover samples) sequentially.
        for sample in remainder {
            *sample = self.run(*sample);
        }
    }

    /// Reset the internal filter state.
    pub const fn reset(&mut self) {
        self.prev = T::ZERO;
    }
}

#[cfg(test)]
mod test_util;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{calculate_power, sine_wave_samples, target_dir_test_artifacts};
    use audio_visualizer::WaveformVisualizer;
    use std::iter;
    use std::vec::Vec;

    #[test]
    fn test_lpf_and_visualize() {
        let sampling_rate = 100.0;
        let cutoff_fr = 10.0;
        let samples_l_orig = sine_wave_samples(15.0, sampling_rate);
        let samples_h_orig = sine_wave_samples(40.0, sampling_rate);

        WaveformVisualizer::new(&samples_l_orig)
            .sample_rate(sampling_rate)
            .y_range(-1.0..1.0)
            .write_png(format!(
                "{}/test_lpf_l_orig.png",
                target_dir_test_artifacts().display()
            ))
            .unwrap();

        WaveformVisualizer::new(&samples_h_orig)
            .sample_rate(sampling_rate)
            .y_range(-1.0..1.0)
            .write_png(format!(
                "{}/test_lpf_h_orig.png",
                target_dir_test_artifacts().display()
            ))
            .unwrap();

        let mut samples_l_lowpassed = samples_l_orig.clone();
        let mut samples_h_lowpassed = samples_h_orig.clone();

        let power_l_orig = calculate_power(&samples_l_orig);
        let power_h_orig = calculate_power(&samples_h_orig);

        let mut filter = LowpassFilter::new(sampling_rate, cutoff_fr);
        filter.run_slice(&mut samples_l_lowpassed);
        filter.reset();
        filter.run_slice(&mut samples_h_lowpassed);

        let power_l_lowpassed = calculate_power(&samples_l_lowpassed);
        let power_h_lowpassed = calculate_power(&samples_h_lowpassed);

        WaveformVisualizer::new(&samples_l_lowpassed)
            .sample_rate(sampling_rate)
            .y_range(-1.0..1.0)
            .write_png(format!(
                "{}/test_lpf_l_after.png",
                target_dir_test_artifacts().display()
            ))
            .unwrap();

        WaveformVisualizer::new(&samples_h_lowpassed)
            .sample_rate(sampling_rate)
            .y_range(-1.0..1.0)
            .write_png(format!(
                "{}/test_lpf_h_after.png",
                target_dir_test_artifacts().display()
            ))
            .unwrap();

        assert!(power_h_lowpassed < power_h_orig);
        assert!(power_l_lowpassed < power_l_orig);

        assert!(
            power_h_lowpassed * 3.0 <= power_l_lowpassed,
            "LPF must actively remove frequencies above threshold"
        );
    }

    /// A lowpass filter lets low frequencies ("bass") pass and removes high
    /// frequencies ("treble"). With a cutoff of 1000 Hz, a 100 Hz tone should
    /// come out almost unchanged, while a 10000 Hz tone should mostly vanish.
    #[test]
    fn test_keeps_low_and_removes_high_frequencies() {
        let sample_rate = 44100.0;
        let cutoff = 1000.0;

        // Loudness of a tone: its highest amplitude. The first 0.1 seconds
        // are skipped, as the filter needs a moment to fade in.
        let loudness = |samples: &[f32]| {
            samples[4410..]
                .iter()
                .fold(0.0_f32, |max, &sample| max.max(sample.abs()))
        };

        let mut low_tone = sine_wave_samples(100.0, sample_rate);
        let mut high_tone = sine_wave_samples(10000.0, sample_rate);
        // Both tones start equally loud.
        assert!(loudness(&low_tone) > 0.99);
        assert!(loudness(&high_tone) > 0.99);

        let mut filter = LowpassFilter::new(sample_rate, cutoff);
        filter.run_slice(&mut low_tone);
        filter.reset();
        filter.run_slice(&mut high_tone);

        assert!(
            loudness(&low_tone) > 0.9,
            "low tone should pass almost unchanged: {}",
            loudness(&low_tone)
        );
        assert!(
            loudness(&high_tone) < 0.2,
            "high tone should be mostly removed: {}",
            loudness(&high_tone)
        );
    }

    /// A lowpass filter smooths a signal: it cannot follow sudden changes
    /// instantly. So when a signal starts abruptly, here jumping from silence
    /// straight to a constant `1.0`, the output must fade in gradually. A
    /// jump in the output would be audible as a "click".
    ///
    /// This is mostly relevant for the special handling of the very first
    /// sample.
    #[test]
    fn test_abrupt_start_fades_in_without_click() {
        let mut samples = [1.0_f32; 20];
        LowpassFilter::new(44100.0, 1000.0).run_slice(&mut samples);

        // The filter starts from silence (0.0), so include it in the output.
        let output = iter::once(0.0).chain(samples).collect::<Vec<_>>();
        let steps = output
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect::<Vec<_>>();

        // Fade in: the output rises towards 1.0 with every sample ...
        assert!(
            steps.iter().all(|&step| step > 0.0),
            "output should rise with every sample: {output:?}"
        );
        // ... and smoothly: each step is smaller than the previous one, so
        // there is no sudden jump at any point.
        assert!(
            steps.windows(2).all(|pair| pair[1] < pair[0]),
            "output should rise in ever smaller steps: {output:?}"
        );
        // ... without ever going beyond the input.
        assert!(
            samples.iter().all(|&sample| sample < 1.0),
            "output should stay below the input: {output:?}"
        );
    }

    /// Tests that the SIMD slice path produces the same results as the
    /// per-sample path, including all tail lengths around the block size.
    #[test]
    fn test_run_slice_matches_run() {
        for n in [0_usize, 1, 3, 7, 8, 9, 16, 17, 41, 1003] {
            let samples_f64 = (0..n)
                .map(|i| (i as f64 * 0.37).sin() * 0.9)
                .collect::<Vec<_>>();
            let samples_f32 = samples_f64.iter().map(|&x| x as f32).collect::<Vec<_>>();

            let mut expected_f32 = samples_f32.clone();
            let mut actual_f32 = samples_f32;
            let mut filter = LowpassFilter::new(44100.0, 120.0);
            for sample in &mut expected_f32 {
                *sample = filter.run(*sample);
            }
            LowpassFilter::new(44100.0, 120.0).run_slice(&mut actual_f32);
            for (i, (e, a)) in expected_f32.iter().zip(&actual_f32).enumerate() {
                assert!((e - a).abs() < 1e-5, "f32, n={n}, i={i}: {e} vs {a}");
            }

            let mut expected_f64 = samples_f64.clone();
            let mut actual_f64 = samples_f64;
            let mut filter = LowpassFilter::new(44100.0, 120.0);
            for sample in &mut expected_f64 {
                *sample = filter.run(*sample);
            }
            LowpassFilter::new(44100.0, 120.0).run_slice(&mut actual_f64);
            for (i, (e, a)) in expected_f64.iter().zip(&actual_f64).enumerate() {
                assert!((e - a).abs() < 1e-12, "f64, n={n}, i={i}: {e} vs {a}");
            }
        }
    }

    /// Tests that the filter state carries over between `run_slice` calls,
    /// so chunked processing equals processing everything at once.
    #[test]
    fn test_run_slice_chunked_equals_whole() {
        let samples = (0..500)
            .map(|i| (i as f32 * 0.37).sin() * 0.9)
            .collect::<Vec<_>>();

        let mut whole = samples.clone();
        let mut filter = LowpassFilter::<f32>::new(44100.0, 120.0);
        filter.run_slice(whole.as_mut_slice());

        let mut chunked = samples;
        let mut filter = LowpassFilter::<f32>::new(44100.0, 120.0);
        // odd chunk size on purpose, so blocks span call boundaries
        for chunk in chunked.chunks_mut(13) {
            filter.run_slice(chunk);
        }

        for (i, (w, c)) in whole.iter().zip(&chunked).enumerate() {
            assert!((w - c).abs() < 1e-5, "i={i}: {w} vs {c}");
        }
    }

    /// Tests that a reset filter behaves like a new one, so it can be reused
    /// for an unrelated signal.
    #[test]
    fn test_reset_equals_new_filter() {
        let samples = (0..100)
            .map(|i| (i as f32 * 0.37).sin() * 0.9)
            .collect::<Vec<_>>();

        let mut expected = samples.clone();
        LowpassFilter::<f32>::new(44100.0, 120.0).run_slice(&mut expected);

        let mut filter = LowpassFilter::<f32>::new(44100.0, 120.0);
        filter.run_slice(&mut [0.9; 50]);
        filter.reset();
        let mut actual = samples;
        filter.run_slice(&mut actual);

        assert_eq!(expected, actual);
    }

    /// Tests if the functions with f32 and f64 behave similar.
    #[test]
    fn test_lpf_f32_f64() {
        let sampling_rate = 44100.0;

        let samples_h_orig = sine_wave_samples(350.0, sampling_rate);
        let mut lowpassed_f32 = samples_h_orig.clone();
        let mut lowpassed_f64 = samples_h_orig.iter().map(|x| *x as f64).collect::<Vec<_>>();

        LowpassFilter::new(sampling_rate, 90.0).run_slice(&mut lowpassed_f32);
        LowpassFilter::new(sampling_rate as f64, 90.0).run_slice(&mut lowpassed_f64);

        let power_f32 = calculate_power(&lowpassed_f32);
        let power_f64 =
            calculate_power(&lowpassed_f64.iter().map(|x| *x as f32).collect::<Vec<_>>());

        assert!((power_f32 - power_f64).abs() <= 0.00024);
    }
}
