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
#[macro_use]
extern crate std;

use audio_visualizer::waveform::staticc::png_file::waveform_static_png_visualize;
use audio_visualizer::{ChannelInterleavement, Channels};
use lowpass_filter::simple::sp::apply_lpf_i16_sp;
use minimp3::{Decoder as Mp3Decoder, Error as Mp3Error, Frame as Mp3Frame};
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let mut path = PathBuf::new();
    path.push("test/samples");
    path.push("sample_1.mp3");
    let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

    let mut lrlr_mp3_samples = vec![];
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame {
                data: samples_of_frame,
                ..
            }) => {
                for sample in samples_of_frame {
                    lrlr_mp3_samples.push(sample);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    // for comparison: visualize original
    waveform_static_png_visualize(
        &lrlr_mp3_samples,
        Channels::Stereo(ChannelInterleavement::LRLR),
        "test/out",
        "sample_1_waveform.png",
    );

    // split into left and right channel
    let (mut left, mut right) = Channels::Stereo(ChannelInterleavement::LRLR)
        .stereo_interleavement()
        .to_channel_data(&lrlr_mp3_samples);

    let now = Instant::now();
    // left
    apply_lpf_i16_sp(&mut left, 44100, 120);
    let then = now.elapsed();
    println!(
        "took {}us to apply low pass filter for left channel ({}) samples",
        then.as_micros(),
        left.len()
    );
    // right
    apply_lpf_i16_sp(&mut right, 44100, 120);

    // visualize audio as waveform in a PNG file
    waveform_static_png_visualize(
        &left,
        Channels::Mono,
        "test/out",
        "sample_1_waveform_lowpassed_left.png",
    );
    waveform_static_png_visualize(
        &right,
        Channels::Mono,
        "test/out",
        "sample_1_waveform_lowpassed_right.png",
    );
}
