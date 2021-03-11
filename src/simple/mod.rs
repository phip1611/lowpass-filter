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
//! Simple first order low pass filter as described here:
//! https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
//!
//! It applies a low pass filter on a vector of samples. It mutates the input array.
//! Therefore, the number of output values equals the number of input values.

use alloc::vec::Vec;

/// Applies a low pass filter on a vector of **mono sample** in 16 bit resolution. If you
/// have stereo data, call this function for each channel, convert it first to mono or do
/// whatever fits your use case.
/// * `data` audio samples
/// * `sample_rate_hz` Sample Rate, e.g. 44100Hz
/// * `cutoff_frequency_hz` upper bound for frequencies to be cutted, e.g. 150Hz
pub fn apply_lpf_i16(data: &mut [i16], sample_rate_hz: u16, cutoff_frequency_hz: u16) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1_f64 / (cutoff_frequency_hz as f64 * 2_f64 * core::f64::consts::PI);
    let dt = 1_f64 / sample_rate_hz as f64;
    let alpha = dt / (rc + dt);

    let mut cpy_orig_data = Vec::with_capacity(data.len());
    cpy_orig_data.extend_from_slice(data);

    data[0] = (alpha * cpy_orig_data[0] as f64) as i16;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        data[i] = (data[i - 1] as f64
            + alpha *
            (cpy_orig_data[i] as f64 - data[i-1] as f64)) as i16;
    }
}

/// Same as [`apply_lpf_i16`] but with i32 audio resolution.
pub fn apply_lpf_i32(data: &mut [i32], sample_rate_hz: u16, cutoff_frequency_hz: u16) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1_f64 / (cutoff_frequency_hz as f64 * 2_f64 * core::f64::consts::PI);
    let dt = 1_f64 / sample_rate_hz as f64;
    let alpha = dt / (rc + dt);

    let mut cpy_orig_data = Vec::with_capacity(data.len());
    cpy_orig_data.extend_from_slice(data);

    data[0] = (alpha * cpy_orig_data[0] as f64) as i32;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        data[i] = (data[i - 1] as f64
            + alpha *
            (cpy_orig_data[i] as f64 - data[i-1] as f64)) as i32;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
    use crate::test::{TEST_SAMPLES_DIR, TEST_OUT_DIR};
    use std::path::PathBuf;
    use std::fs::File;
    use audio_visualizer::{Channels, ChannelInterleavement};
    use audio_visualizer::waveform::staticc::png_file::visualize;
    use std::time::Instant;

    /// To see if the test actually works, check the waveform in the image output.
    #[test]
    fn test_visualize_lowpassed_data() {
        let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
        path.push("sample_1.mp3");
        let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

        let mut lrlr_mp3_samples = vec![];
        loop {
            match decoder.next_frame() {
                Ok(Mp3Frame { data: samples_of_frame, .. }) => {
                    for sample in samples_of_frame {
                        lrlr_mp3_samples.push(sample);
                    }
                }
                Err(Mp3Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
        }

        // for comparison: visualize original
        visualize(
            &lrlr_mp3_samples,
            Channels::Stereo(ChannelInterleavement::LRLR),
            TEST_OUT_DIR,
            "sample_1_waveform.png"
        );

        // split into left and right channel
        let (mut left, mut right) = Channels::Stereo(ChannelInterleavement::LRLR)
            .stereo_interleavement()
            .to_channel_data(&lrlr_mp3_samples);

        let now = Instant::now();
        // left
        apply_lpf_i16(&mut left, 44100, 120);
        let then = now.elapsed();
        println!("took {}us to apply low pass filter for left channel ({}) samples", then.as_micros(), left.len());
        // right
        apply_lpf_i16(&mut right, 44100, 120);

        // visualize audio as waveform in a PNG file
        visualize(
            &left,
            Channels::Mono,
            TEST_OUT_DIR,
            "sample_1_waveform_lowpassed_left.png"
        );
        visualize(
            &right,
            Channels::Mono,
            TEST_OUT_DIR,
            "sample_1_waveform_lowpassed_right.png"
        );
    }
}
