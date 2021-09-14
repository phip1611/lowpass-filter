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
//! Minimal example how to use this crate/how to apply low pass filter.
extern crate std;

use audio_visualizer::waveform::staticc::png_file::waveform_static_png_visualize;
use audio_visualizer::{ChannelInterleavement, Channels};
use lowpass_filter::simple::sp::apply_lpf_i16_sp;

/// Minimal example how to use this crate/how to apply low pass filter.
fn main() {
    // read this from MP3 for example
    let audio_data_lrlr = [0_i16, 1, -5, 1551, 141, 24];

    // split into left and right channel
    let (mut left, mut right) = Channels::Stereo(ChannelInterleavement::LRLR)
        .stereo_interleavement()
        .to_channel_data(&audio_data_lrlr);

    // left
    apply_lpf_i16_sp(&mut left, 44100, 120);
    // right
    apply_lpf_i16_sp(&mut right, 44100, 120);

    // visualize effect as waveform in a PNG file
    waveform_static_png_visualize(
        &left,
        Channels::Mono,
        "test/out",
        "example_waveform_lowpassed_left.png",
    );
    waveform_static_png_visualize(
        &right,
        Channels::Mono,
        "test/out",
        "example_waveform_lowpassed_right.png",
    );
}
