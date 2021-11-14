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
//! This is a `no_std` Rust library for simple digital low pass filters. You can use it for
//! example to get the low frequencies from a song.
//!
//! **⚠ Prefer crate `biquad` and use this crate only for educational purposes.**
//!
//! # Example
//! ```ignore
//! use lowpass_filter::lowpass_filter;
//!
//! /// Minimal example how to use this crate/how to apply low pass filter.
//! fn main() {
//!     // read this from MP3 for example
//!     let mut mono_audio_data = [0.0, 1.0, -5.0, 1551.0, 141.0, 24.0];
//!     // mutates the input buffer
//!     lowpass_filter(&mut mono_audio_data, 44100.0, 120.0);
//! }
//! ```

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
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
// use std in tests
#![cfg_attr(not(test), no_std)]

// use alloc crate, because this is no_std
extern crate alloc;

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

/// Applies a single-order lowpass filter with single precisioun to the data provided in the mutable buffer.
pub fn lowpass_filter(data: &mut [f32], sampling_rate: f32, cutoff_frequency: f32) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1.0 / (cutoff_frequency * 2.0 * core::f32::consts::PI);
    // time per sample
    let dt = 1.0 / sampling_rate;
    let alpha = dt / (rc + dt);

    data[0] *= alpha;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter

        // we don't need a copy of the original data, because the original data is accessed
        // before it is overwritten: data[i] = ... data[i]
        data[i] = data[i - 1] + alpha * (data[i] - data[i - 1]);
    }
}

/// Applies a single-order lowpass filter with double precision to the data provided in the mutable buffer.
pub fn lowpass_filter_f64(data: &mut [f64], sampling_rate: f64, cutoff_frequency: f64) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1.0 / (cutoff_frequency * 2.0 * core::f64::consts::PI);
    // time per sample
    let dt = 1.0 / sampling_rate;
    let alpha = dt / (rc + dt);

    data[0] *= alpha;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter

        // we don't need a copy of the original data, because the original data is accessed
        // before it is overwritten: data[i] = ... data[i]
        data[i] = data[i - 1] + alpha * (data[i] - data[i - 1]);
    }
}

#[cfg(test)]
mod test_util;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{calculate_power, sine_wave_samples, TEST_OUT_DIR};
    use audio_visualizer::waveform::plotters_png_file::waveform_static_plotters_png_visualize;
    use audio_visualizer::Channels;
    use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F64};
    use std::time::Instant;
    use std::vec::Vec;

    #[test]
    fn test_lpf_and_visualize() {
        let samples_l_orig = sine_wave_samples(120.0, 44100.0);
        let samples_h_orig = sine_wave_samples(350.0, 44100.0);

        waveform_static_plotters_png_visualize(
            &samples_l_orig.iter().map(|x| *x as i16).collect::<Vec<_>>(),
            Channels::Mono,
            TEST_OUT_DIR,
            "test_lpf_l_orig.png",
        );
        waveform_static_plotters_png_visualize(
            &samples_h_orig.iter().map(|x| *x as i16).collect::<Vec<_>>(),
            Channels::Mono,
            TEST_OUT_DIR,
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
            TEST_OUT_DIR,
            "test_lpf_l_after.png",
        );
        waveform_static_plotters_png_visualize(
            &samples_h_lowpassed
                .iter()
                .map(|x| *x as i16)
                .collect::<Vec<_>>(),
            Channels::Mono,
            TEST_OUT_DIR,
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

        assert!((power_f32 as f64 - power_f64).abs() <= 0.00024);
    }

    #[ignore]
    #[test]
    fn test_lowpass_filter_vs_biquad() {
        let samples_orig = sine_wave_samples(350.0, 44100.0);
        let mut samples = samples_orig.clone();

        let now = Instant::now();
        for _ in 0..1000 {
            lowpass_filter_f64(samples.as_mut_slice(), 44100.0, 90.0);
        }
        let duration_lowpass_filter = now.elapsed().as_secs_f64() / 1000.0;

        let duration_biquad = {
            let f0 = 80.hz();
            let fs = 44.1.khz();

            // Create coefficients for the biquads
            let coeffs =
                Coefficients::<f64>::from_params(Type::LowPass, fs, f0, Q_BUTTERWORTH_F64).unwrap();
            let mut lowpassed_data = Vec::with_capacity(samples_orig.len());
            let mut biquad_lpf = DirectForm1::<f64>::new(coeffs);

            let now = Instant::now();
            for _ in 0..1000 {
                samples_orig
                    .iter()
                    .for_each(|val| lowpassed_data.push(biquad_lpf.run(*val)));
            }

            now.elapsed().as_secs_f64() / 1000.0
        };

        println!(
            "lowpass filter on average: {}µs",
            duration_lowpass_filter * 1_000_000.0
        );
        println!(
            "biquad filter on average : {}µs",
            duration_biquad * 1_000_000.0
        );
    }
}
