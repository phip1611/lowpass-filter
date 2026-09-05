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
//! Samples must be in range `-1.0..=1.0`, which is the default in DSP.
//!
//! ## Usage
//!
//! To filter a buffer of samples in one go, use [`lowpass_filter_slice`]
//! (or [`lowpass_filter_slice_f64`]). This is the fastest option: it
//! processes samples in blocks that compilers auto-vectorize (SIMD),
//! several times faster than per-sample processing.
//!
//! ```rust
//! use lowpass_filter::lowpass_filter_slice;
//!
//! // Mono audio samples, recorded at 44.1 kHz sample rate.
//! let mut samples = [0.0, 0.3, -0.6, 0.8, 0.5, -0.2];
//! // Only keep frequencies below 120 Hz; mutates the buffer in-place.
//! lowpass_filter_slice(&mut samples, 44100.0, 120.0);
//! ```
//!
//! For streaming data, e.g. in an audio callback, keep a [`LowpassFilter`]
//! around: its state carries over between calls, so chunked processing
//! equals processing everything at once. It also filters single samples,
//! e.g. inside iterator chains.
//!
//! ```rust
//! use lowpass_filter::LowpassFilter;
//!
//! let mut filter = LowpassFilter::<f32>::new(44100.0, 120.0);
//! // Process data as it arrives (fast block processing) ...
//! for mut chunk in [[0.0, 0.3, -0.6, 0.8], [0.5, -0.2, 0.1, 0.4]] {
//!     filter.run_slice(&mut chunk);
//! }
//! // ... or one sample at a time.
//! let filtered = filter.run(0.25);
//! ```
//!
//! The iterator-based [`lowpass_filter`] and [`lowpass_filter_f64`]
//! functions are convenient when the samples do not live in a slice.

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
use core::ops::{Add, AddAssign, Div, Mul, Neg, RangeInclusive, Sub};

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

    /// See [`f32::clamp`].
    #[must_use]
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl Sample for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const PI: Self = core::f32::consts::PI;

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        Self::clamp(self, min, max)
    }
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const PI: Self = core::f64::consts::PI;

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        Self::clamp(self, min, max)
    }
}

/// A first-order lowpass filter compatible with `f32` and `f64`.
///
/// It can consume and filter items one-by-one (iterator-style API) or operate
/// on slices ([`LowpassFilter::run_slice`]).
///
/// It is mandatory to operate on values in range `-1.0..=1.0`, which is also
/// the default in DSP.
///
/// # More Info
/// - <https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter>
#[derive(Debug, Clone)]
pub struct LowpassFilter<T> {
    alpha: T,
    /// Precomputed `1 - alpha`.
    beta: T,
    prev: T,
    next_is_first: bool,
}

impl<T: Sample> LowpassFilter<T> {
    /// Create a new lowpass filter.
    ///
    /// # Arguments
    /// - `sample_rate_hz`: Sample rate in Hz (e.g., 48000.0).
    /// - `cutoff_frequency_hz`: Cutoff frequency in Hz (e.g., 1000.0).
    #[must_use]
    pub fn new(sample_rate_hz: T, cutoff_frequency_hz: T) -> Self {
        // Nyquist rule
        assert!(cutoff_frequency_hz * T::TWO <= sample_rate_hz);

        let rc = T::ONE / (cutoff_frequency_hz * T::TWO * T::PI);
        let dt = T::ONE / sample_rate_hz;
        let alpha = dt / (rc + dt);

        Self {
            alpha,
            beta: T::ONE - alpha,
            prev: T::ZERO,
            next_is_first: true,
        }
    }

    /// Filter a single sample and return the filtered result.
    ///
    /// It is mandatory to operate on values in range `-1.0..=1.0`, which is
    /// also the default in DSP. The returned value is also guaranteed to be in
    /// that range.
    #[inline]
    pub fn run(&mut self, input: T) -> T {
        let range: RangeInclusive<T> = -T::ONE..=T::ONE;
        debug_assert!(
            range.contains(&input),
            "samples must be in range {range:?}: {input}"
        );

        let value = if self.next_is_first {
            self.next_is_first = false;
            self.prev = input;
            input * self.alpha
        } else {
            // Re-associated form of `prev + alpha * (input - prev)`:
            self.prev = self.alpha * input + self.beta * self.prev;
            self.prev
        };

        // very small deviations caused by floating point operations
        // are tolerable; just truncate the value
        value.clamp(-T::ONE, T::ONE)
    }

    /// Filter a whole slice of samples in-place.
    ///
    /// Matches calling [`Self::run`] per sample up to tiny floating
    /// point rounding differences (roughly `1e-6` for `f32`), but
    /// is significantly faster. The filter state is updated, so
    /// consecutive calls compose, also when mixed with
    /// [`Self::run`].
    ///
    /// # Math
    /// Unrolling `y[n] = alpha * x[n] + beta * y[n-1]` over a block
    /// of samples yields
    ///
    /// ```text
    /// y[i] = beta^(i+1) * prev + sum(alpha * beta^(i-j) * x[j] for j <= i)
    /// ```
    ///
    /// so within a block, samples only depend on the state `prev`
    /// from before the block and can be computed in parallel, which
    /// enables compiler auto-vectorization (SIMD). Only `prev`
    /// propagates serially between blocks.
    ///
    /// # Arguments
    /// - `samples`: Samples to filter in-place, in range `-1.0..=1.0`.
    pub fn run_slice(&mut self, samples: &mut [T]) {
        // Block size. 8 measured fastest on x86-64 for f32 and f64.
        const LANES: usize = 8;

        let mut samples = samples;
        // The first sample is special-cased in `run`; handle it
        // there so the block form below is uniform.
        if self.next_is_first {
            if let Some((first, rest)) = samples.split_first_mut() {
                *first = self.run(*first);
                samples = rest;
            } else {
                return;
            }
        }

        // Coefficients of the closed block form (see doc comment):
        //   y[i] = beta^(i+1) * prev + sum(alpha * beta^(i-j) * x[j] for j <= i)
        // pow[k] = beta^k
        let mut pow = [T::ONE; LANES];
        for k in 1..LANES {
            pow[k] = pow[k - 1] * self.beta;
        }
        // carry_coeffs[i] = beta^(i+1), the weight of `prev` in y[i]
        let carry_coeffs = pow.map(|p| p * self.beta);

        // cols[j][i] = alpha * beta^(i-j): the weight of input x[j]
        // in output y[i], stored as one "column" per input j so
        // that the hot loop below can apply one sample to all
        // outputs at once. Entries for i < j stay 0, as later
        // inputs cannot affect earlier outputs.
        let mut cols = [[T::ZERO; LANES]; LANES];
        for (j, col) in cols.iter_mut().enumerate() {
            for (i, weight) in col.iter_mut().enumerate().skip(j) {
                *weight = self.alpha * pow[i - j];
            }
        }

        // Hot loop. `acc[i]` accumulates y[i] of the current block.
        // Its shape helps the compilers auto-vectorizer.
        let (chunks, remainder) = samples.as_chunks_mut::<LANES>();
        for chunk in chunks {
            let mut acc = [T::ZERO; LANES];
            // acc[i] = sum(cols[j][i] * x[j] for all j)
            for (col, &sample) in cols.iter().zip(chunk.iter()) {
                for (acc, &coeff) in acc.iter_mut().zip(col.iter()) {
                    *acc += coeff * sample;
                }
            }
            // acc[i] += beta^(i+1) * prev; the only place where
            // state from before the block enters.
            for (acc, &coeff) in acc.iter_mut().zip(carry_coeffs.iter()) {
                *acc += coeff * self.prev;
            }
            // like in `run`, `prev` keeps the unclamped value
            self.prev = acc[LANES - 1];
            for (sample, acc) in chunk.iter_mut().zip(acc.iter()) {
                *sample = acc.clamp(-T::ONE, T::ONE);
            }
        }
        // Process the up to LANES - 1 leftover samples sequentially.
        for sample in remainder {
            *sample = self.run(*sample);
        }
    }

    /// Reset the internal filter state.
    pub const fn reset(&mut self) {
        self.prev = T::ZERO;
        self.next_is_first = true;
    }
}

/// Applies a [`LowpassFilter`] to the data provided in the mutable buffer and
/// changes the items in-place.
///
/// It is mandatory to operate on f32 values in range `-1.0..=1.0`, which is
/// also the default in DSP.
///
/// # Arguments
/// - `sample_iter`: Iterator over the samples. This can also be a
///   `[1.0, ...]`-style slice
/// - `sample_rate_hz`: Sample rate in Hz (e.g., 48000.0).
/// - `cutoff_frequency_hz`: Cutoff frequency in Hz (e.g., 1000.0).
#[inline]
pub fn lowpass_filter<'a, I: IntoIterator<Item = &'a mut f32>>(
    sample_iter: I,
    sample_rate_hz: f32,
    cutoff_frequency_hz: f32,
) {
    let mut filter = LowpassFilter::<f32>::new(sample_rate_hz, cutoff_frequency_hz);

    for sample in sample_iter.into_iter() {
        let new_sample = filter.run(*sample);
        *sample = new_sample;
    }
}

/// Applies a [`LowpassFilter`] to the data provided in the mutable buffer and
/// changes the items in-place.
///
/// It is mandatory to operate on f64 values in range `-1.0..=1.0`, which is
/// also the default in DSP.
///
/// # Arguments
/// - `sample_iter`: Iterator over the samples. This can also be a
///   `[1.0, ...]`-style slice
/// - `sample_rate_hz`: Sample rate in Hz (e.g., 48000.0).
/// - `cutoff_frequency_hz`: Cutoff frequency in Hz (e.g., 1000.0).
#[inline]
pub fn lowpass_filter_f64<'a, I: IntoIterator<Item = &'a mut f64>>(
    sample_iter: I,
    sample_rate_hz: f64,
    cutoff_frequency_hz: f64,
) {
    let mut filter = LowpassFilter::<f64>::new(sample_rate_hz, cutoff_frequency_hz);

    for sample in sample_iter.into_iter() {
        let new_sample = filter.run(*sample);
        *sample = new_sample;
    }
}

/// Applies a [`LowpassFilter`] to the slice in-place via
/// [`LowpassFilter::run_slice`].
///
/// Significantly faster than [`lowpass_filter`], with results equal up to
/// tiny floating point rounding differences (roughly `1e-6`).
///
/// It is mandatory to operate on f32 values in range `-1.0..=1.0`, which is
/// also the default in DSP.
///
/// # Arguments
/// - `samples`: Samples to filter in-place.
/// - `sample_rate_hz`: Sample rate in Hz (e.g., 48000.0).
/// - `cutoff_frequency_hz`: Cutoff frequency in Hz (e.g., 1000.0).
#[inline]
pub fn lowpass_filter_slice(samples: &mut [f32], sample_rate_hz: f32, cutoff_frequency_hz: f32) {
    let mut filter = LowpassFilter::<f32>::new(sample_rate_hz, cutoff_frequency_hz);
    filter.run_slice(samples);
}

/// Applies a [`LowpassFilter`] to the slice in-place via
/// [`LowpassFilter::run_slice`].
///
/// Significantly faster than [`lowpass_filter_f64`], with results equal up
/// to tiny floating point rounding differences.
///
/// It is mandatory to operate on f64 values in range `-1.0..=1.0`, which is
/// also the default in DSP.
///
/// # Arguments
/// - `samples`: Samples to filter in-place.
/// - `sample_rate_hz`: Sample rate in Hz (e.g., 48000.0).
/// - `cutoff_frequency_hz`: Cutoff frequency in Hz (e.g., 1000.0).
#[inline]
pub fn lowpass_filter_slice_f64(
    samples: &mut [f64],
    sample_rate_hz: f64,
    cutoff_frequency_hz: f64,
) {
    let mut filter = LowpassFilter::<f64>::new(sample_rate_hz, cutoff_frequency_hz);
    filter.run_slice(samples);
}

#[cfg(test)]
mod test_util;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{calculate_power, sine_wave_samples, target_dir_test_artifacts};
    use audio_visualizer::Channels;
    use audio_visualizer::waveform::plotters_png_file::waveform_static_plotters_png_visualize;
    use std::vec::Vec;

    #[test]
    fn test_lpf_and_visualize() {
        let samples_l_orig = sine_wave_samples(120.0, 44100.0);
        let samples_h_orig = sine_wave_samples(350.0, 44100.0);

        waveform_static_plotters_png_visualize(
            &samples_l_orig.iter().map(|x| *x as i16).collect::<Vec<_>>(),
            Channels::Mono,
            target_dir_test_artifacts().to_str().unwrap(),
            "test_lpf_l_orig.png",
        );
        waveform_static_plotters_png_visualize(
            &samples_h_orig.iter().map(|x| *x as i16).collect::<Vec<_>>(),
            Channels::Mono,
            target_dir_test_artifacts().to_str().unwrap(),
            "test_lpf_h_orig.png",
        );

        let mut samples_l_lowpassed = samples_l_orig.clone();
        let mut samples_h_lowpassed = samples_h_orig.clone();

        let power_l_orig = calculate_power(&samples_l_orig);
        let power_h_orig = calculate_power(&samples_h_orig);

        lowpass_filter_f64(samples_l_lowpassed.as_mut_slice(), 44100.0, 90.0);
        lowpass_filter_f64(samples_h_lowpassed.as_mut_slice(), 44100.0, 90.0);

        let power_l_lowpassed = calculate_power(&samples_l_lowpassed);
        let power_h_lowpassed = calculate_power(&samples_h_lowpassed);

        waveform_static_plotters_png_visualize(
            &samples_l_lowpassed
                .iter()
                .map(|x| *x as i16)
                .collect::<Vec<_>>(),
            Channels::Mono,
            target_dir_test_artifacts().to_str().unwrap(),
            "test_lpf_l_after.png",
        );
        waveform_static_plotters_png_visualize(
            &samples_h_lowpassed
                .iter()
                .map(|x| *x as i16)
                .collect::<Vec<_>>(),
            Channels::Mono,
            target_dir_test_artifacts().to_str().unwrap(),
            "test_lpf_h_after.png",
        );

        assert!(power_h_lowpassed < power_h_orig);
        assert!(power_l_lowpassed < power_l_orig);

        assert!(
            power_h_lowpassed * 3.0 <= power_l_lowpassed,
            "LPF must actively remove frequencies above threshold"
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
            let mut actual_f32 = samples_f32.clone();
            lowpass_filter(expected_f32.as_mut_slice(), 44100.0, 120.0);
            lowpass_filter_slice(actual_f32.as_mut_slice(), 44100.0, 120.0);
            for (i, (e, a)) in expected_f32.iter().zip(&actual_f32).enumerate() {
                assert!((e - a).abs() < 1e-5, "f32, n={n}, i={i}: {e} vs {a}");
            }

            let mut expected_f64 = samples_f64.clone();
            let mut actual_f64 = samples_f64.clone();
            lowpass_filter_f64(expected_f64.as_mut_slice(), 44100.0, 120.0);
            lowpass_filter_slice_f64(actual_f64.as_mut_slice(), 44100.0, 120.0);
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

    /// Tests if the functions with f32 and f64 behave similar.
    #[test]
    fn test_lpf_f32_f64() {
        let samples_h_orig = sine_wave_samples(350.0, 44100.0);
        let mut lowpassed_f32 = samples_h_orig.iter().map(|x| *x as f32).collect::<Vec<_>>();
        #[allow(clippy::redundant_clone)]
        let mut lowpassed_f64 = samples_h_orig.clone();

        lowpass_filter(lowpassed_f32.as_mut_slice(), 44100.0, 90.0);
        lowpass_filter_f64(lowpassed_f64.as_mut_slice(), 44100.0, 90.0);

        let power_f32 =
            calculate_power(&lowpassed_f32.iter().map(|x| *x as f64).collect::<Vec<_>>());
        let power_f64 = calculate_power(&lowpassed_f64);

        assert!((power_f32 - power_f64).abs() <= 0.00024);
    }
}
