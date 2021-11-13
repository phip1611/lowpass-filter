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

use std::f64::consts::PI;
use std::vec::Vec;

/// If tests create files, they should be stored here.
pub const TEST_OUT_DIR: &str = "test/out";

pub fn sine_wave(fr: f64) -> impl Fn(f64) -> f64 {
    move |time| (2.0 * PI * fr * time).sin()
}

pub fn sine_wave_samples(fr: f64, sampling_rate: f64) -> Vec<f64> {
    let sine_wave = sine_wave(fr);
    // 2 seconds long
    (0..(2 * sampling_rate as usize))
        .map(|x| x as f64)
        .map(|t| t / sampling_rate)
        .map(|t| sine_wave(t) * i16::MAX as f64)
        .collect::<Vec<_>>()
}

pub fn calculate_power(samples: &[f64]) -> f64 {
    samples
        .iter()
        .copied()
        .map(|x| x / i16::MAX as f64)
        .map(|x| x * x)
        .fold(0.0, |acc, val| acc + val)
}
