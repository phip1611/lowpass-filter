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

#![allow(unused)]

use hound::{SampleFormat, WavSpec};
use itertools::Itertools;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};
use std::vec::Vec;

/// Reads a WAV file to mono audio. Returns the samples as mono audio.
/// Additionally, it returns the sampling rate of the file.
pub fn read_wav_to_mono<T: AsRef<Path>>(file: T) -> (Vec<i16>, WavSpec) {
    let mut reader = hound::WavReader::open(file).unwrap();
    let header = reader.spec();

    // owning vector with original data in i16 format
    let data = reader
        .samples::<i16>()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    if header.channels == 1 {
        (data, header)
    } else if header.channels == 2 {
        let data = data
            .into_iter()
            .chunks(2)
            .into_iter()
            .map(|mut lr| {
                let l = lr.next().unwrap();
                let r = lr
                    .next()
                    .expect("should have an even number of LRLR samples");
                stereo_to_mono(l, r)
            })
            .collect::<Vec<_>>();
        (data, header)
    } else {
        panic!("unsupported format!");
    }
}

/// Writes a WAV file as mono.
pub fn write_wav_file(path: &Path, samples: &[i16], sample_rate: u32) {
    let mut wav_writer = hound::WavWriter::create(
        path,
        WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        },
    )
    .unwrap();

    for &sample in samples {
        wav_writer.write_sample(sample).unwrap()
    }
    wav_writer.finalize().unwrap();
}

/// Transforms two stereo samples (that reflect the same point in time on
/// different channels) into one mono sample.
#[inline]
#[must_use]
pub const fn stereo_to_mono(l: i16, r: i16) -> i16 {
    let l = l as i32;
    let r = r as i32;
    let avg = (l + r) / 2;
    avg as i16
}

/// Transforms an audio sample in range `i16::MIN..=i16::MAX` to a `f32` in
/// range `-1.0..=1.0`.
#[inline]
pub fn i16_sample_to_f32(val: i16) -> f32 {
    // If to prevent division result >1.0.
    if val == i16::MIN {
        -1.0
    } else {
        val as f32 / i16::MAX as f32
    }
}

/// Transforms an audio sample of type `f32` in range `-1.0..=1.0` to a `i16` in
/// range `-i16::MAX..=i16::MAX`.
#[inline]
pub fn f32_sample_to_i16(val: f32) -> i16 {
    if val.is_finite() && val.abs() <= 1.0 {
        (val * i16::MAX as f32) as i16
    } else {
        panic!("invalid f32");
    }
}

/// Returns the cargo target dir.
pub fn target_dir() -> PathBuf {
    // 1. Check if CARGO_TARGET_DIR is set
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
    } else {
        // 2. Fall back to default: go up from CARGO_MANIFEST_DIR
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("target")
    }
}
/// Returns a directory within the cargo target dir to store test artifacts.
pub fn target_dir_test_artifacts() -> PathBuf {
    let mut path = target_dir();
    path.push("test_generated");

    // create if not exists
    if !fs::exists(&path).unwrap() {
        fs::create_dir(&path).unwrap();
    }

    path
}

pub fn sine_wave(fr: f64) -> impl Fn(f64) -> f64 {
    move |time| (2.0 * PI * fr * time).sin()
}

/// Creates a two second long audio snippet from the given frequency.
pub fn sine_wave_samples(fr: f64, sampling_rate: f64) -> Vec<f64> {
    let sine_wave = sine_wave(fr);
    // 2 seconds long
    (0..(2 * sampling_rate as usize))
        .map(|x| x as f64)
        .map(|t| t / sampling_rate)
        .map(|t| sine_wave(t))
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
