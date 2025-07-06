/*
MIT License

Copyright (c) 2021 Philipp Schuster

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
//! Simple first-order digital lowpass filters, compatible with `no_std`. You can
//! use it, for example, to get the low frequencies from a song.
//!
//! ## Difference to `biquad`
//!
//! **⚠ TL;DR: `biquad` might be a better option in some use-cases.**
//!
//! This crate provides a basic and simple to understand, first order lowpass
//! filter. The [biquad](https://crates.io/crates/biquad) crate offers second order
//! filters, with higher accuracy. From my testing, a lowpass filter created with
//! `biquad` has higher computational costs as this crate, but offers a
//! **better resolution for actually cutting of signals above the cut-off frequency
//! while the preserved signal will be less attenuated**.
//!
//! So for production use-cases, please also consider using `biquad`. You can run
//! benchmark and check your data (e.g., by plotting) to make that decision.
//!
//! ## Usage
//!
//! You can either use the [`LowpassFilter`] type to integrate the filter in
//! iterator chains or you can use a convenient function such as
//! [`lowpass_filter`] and [`lowpass_filter_f64`]. The first approach is more
//! flexible.
//!
//! # Example
//! ```rust,no_run
//! use lowpass_filter::lowpass_filter;
//!
//! // some samples
//! let mut mono_audio_data = [0.0, 1.0, -5.0, 1551.0, 141.0, 24.0];
//! // mutates the input buffer
//! lowpass_filter(&mut mono_audio_data, 44100.0, 120.0);
//! ```

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

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

use std::ops::RangeInclusive;

/// A single-order lowpass filter with single precision that consumes and emits
/// items one by one.
///
/// It is mandatory to operate on f32 values in range `-1.0..=1.0`, which is
/// also the default in DSP.
///
/// # More Info
/// - <https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter>
#[derive(Debug, Clone)]
pub struct LowpassFilter<T> {
    alpha: T,
    prev: T,
    next_is_first: bool,
}

macro_rules! impl_lowpass_filter {
    ($t:ty, $pi:expr) => {
        impl LowpassFilter<$t> {
            /// Create a new lowpass filter.
            ///
            /// # Arguments
            /// - `sample_rate_hz`: Sample rate in Hz (e.g., 48000.0).
            /// - `cutoff_frequency_hz`: Cutoff frequency in Hz (e.g., 1000.0).
            #[must_use]
            pub fn new(sample_rate_hz: $t, cutoff_frequency_hz: $t) -> Self {
                // Nyquist rule
                assert!(cutoff_frequency_hz * 2.0 <= sample_rate_hz);

                let rc = 1.0 / (cutoff_frequency_hz * 2.0 * $pi);
                let dt = 1.0 / sample_rate_hz;
                let alpha = dt / (rc + dt);

                Self {
                    alpha,
                    prev: 0.0,
                    next_is_first: true,
                }
            }

            /// Filter a single sample and return the filtered result.
            ///
            /// It is mandatory to operate on f32 values in range
            /// `-1.0..=1.0`, which is also the default in DSP. The returned
            /// value is also guaranteed to be in that range.
            #[inline]
            pub fn run(&mut self, input: $t) -> $t {
                const RANGE: RangeInclusive<$t> = -1.0..=1.0;
                debug_assert!(
                    RANGE.contains(&input),
                    "samples must be in range {RANGE:?}: {input}"
                );

                let value = if self.next_is_first {
                    self.next_is_first = false;
                    self.prev = input;
                    input * self.alpha
                } else {
                    self.prev = self.prev + self.alpha * (input - self.prev);
                    self.prev
                };

                // very small deviations caused by floating point operations
                // are tolerable; just truncate the value
                value.clamp(-1.0, 1.0)
            }

            /// Reset the internal filter state.
            pub const fn reset(&mut self) {
                self.prev = 0.0;
                self.next_is_first = true;
            }
        }
    };
}

impl_lowpass_filter!(f32, core::f32::consts::PI);
impl_lowpass_filter!(f64, core::f64::consts::PI);

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
/// It is mandatory to operate on f32 values in range `-1.0..=1.0`, which is
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

        assert!(power_h_lowpassed <= 3.0 * power_h_lowpassed);
        assert!(
            power_h_lowpassed * 3.0 <= power_l_lowpassed,
            "LPF must actively remove frequencies above threshold"
        );
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
