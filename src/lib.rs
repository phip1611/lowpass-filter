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

// use std in tests
#![cfg_attr(not(test), no_std)]

// use alloc crate, because this is no_std
extern crate alloc;

#[allow(unused)]
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

pub mod simple;

mod num_traits;
#[cfg(test)]
mod test_util;

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::f64::consts::PI;
    use wav::{Header, BitDepth};
    use std::fs::File;
    use audio_visualizer::waveform::staticc::plotters_png_file::waveform_static_plotters_png_visualize;
    use audio_visualizer::Channels;
    use crate::simple::{Filter, FirstOrderLowPassFilterTrait};
    use crate::test_util::TEST_OUT_DIR;

    fn sine_wave(fr: f64) -> impl Fn(f64) -> f64 {
        move |time| (2.0 * PI * fr * time).sin()
    }

    fn sine_wave_samples(fr: u32, sampling_rate: u32) -> Vec<i16> {
        let sine_wave = sine_wave(fr as f64);
        // 2 seconds long
        (0..(2 * sampling_rate))
            .map(|x| x as f64)
            .map(|t| t / sampling_rate as f64)
            .map(|t| sine_wave(t) * i16::MAX as f64)
            .map(|x| x as i16)
            .collect::<Vec<_>>()
    }

    fn calculate_power(samples: &[i16]) -> u128 {
        let data = samples.iter()
            .map(|x| *x as f64)
            .map(|x| x / i16::MAX as f64)
            .map(|x| x * x)
            .collect::<Vec<f64>>();

        let mut sum = 0.0;
        for x in data {
            sum += x;
        }

        sum as u128
    }

    #[test]
    fn test_lpf() {
        let samples_l = sine_wave_samples(120, 44100);
        let samples_h = sine_wave_samples(250, 44100);

        let _power_l_before: u128 = calculate_power(&samples_l);
        let _power_h_before: u128 = calculate_power(&samples_h);


        /*waveform_static_plotters_png_visualize(
            &samples_l,
            Channels::Mono,
            TEST_OUT_DIR,
            "test_lpf_l_before.png"
        );
        waveform_static_plotters_png_visualize(
            &samples_h,
            Channels::Mono,
            TEST_OUT_DIR,
            "test_lpf_h_before.png"
        );*/

        let samples_l = Filter::<f32>::apply(&samples_l, 44100, 90);
        let samples_h = Filter::<f32>::apply(&samples_h, 44100, 90);

        let power_l_after = calculate_power(&samples_l);
        let power_h_after = calculate_power(&samples_h);

        assert!(power_h_after * 3 <= power_l_after, "LPF must actively remove frequencies above threshold");

        /*waveform_static_plotters_png_visualize(
            &samples_l,
            Channels::Mono,
            TEST_OUT_DIR,
            "test_lpf_l_after.png"
        );
        waveform_static_plotters_png_visualize(
            &samples_h,
            Channels::Mono,
            TEST_OUT_DIR,
            "test_lpf_h_after.png"
        );*/
    }


}
