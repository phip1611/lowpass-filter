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

/// Single precision / f32 / float.
pub mod sp;
/// Double precision / f64 / double.
pub mod dp;

#[cfg(test)]
mod tests {
    use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
    use std::path::PathBuf;
    use std::fs::File;
    use audio_visualizer::{Channels, ChannelInterleavement};
    use audio_visualizer::waveform::staticc::png_file::visualize;
    use std::time::Instant;
    use crate::simple::sp::apply_lpf_i16_sp;
    use crate::simple::dp::apply_lpf_i16_dp;
    use crate::test_util::{TEST_OUT_DIR, TEST_SAMPLES_DIR};

    /// To see if the test actually works, check the waveform in the image output.
    #[test]
    fn test_visualize_lowpassed_data() {
        let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
        path.push("sample_1");
        println!("ENV pwd: {:#?}", std::env::current_dir().unwrap());
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
        for _ in 0..3 {
            apply_lpf_i16_sp(&mut left, 44100, 120);
        }
        let then = now.elapsed();
        println!("took {}us to apply low pass filter for left channel ({}) samples", then.as_micros(), left.len());
        // right
        apply_lpf_i16_sp(&mut right, 44100, 120);

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

    /// To see if the test actually works, check the waveform in the image output.
    #[test]
    fn test_compare_sp_dp_lpf() {
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

        // split into left and right channel
        let (mut left, mut right) = Channels::Stereo(ChannelInterleavement::LRLR)
            .stereo_interleavement()
            .to_channel_data(&lrlr_mp3_samples);

        let sp_now = Instant::now();
        apply_lpf_i16_sp(&mut left, 44100, 120);

        let sp_duration = sp_now.elapsed().as_micros();
        let dp_now = Instant::now();

        apply_lpf_i16_dp(&mut right, 44100, 120);
        let dp_duration = dp_now.elapsed().as_micros();

        println!("sp lpf took: {}µs", sp_duration);
        println!("dp lpf took: {}µs", dp_duration);

        // on x86_64/i7-10600K I experienced that both are equally fast IN DEBUG MODE
        // in Release mode SP is of course a few percent faster
    }
}
