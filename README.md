# Rust: `no_std` digital low pass filter library
This is a `no_std` Rust library for simple digital low pass filters. You can use it for example to 
get the low frequencies from a song.

**I'm not an expert on digital signal processing. Code contributions are highly welcome! :)**

## How to use
```rust
use audio_visualizer::waveform::staticc::png_file::waveform_static_png_visualize;
// audio_visualizer has some cool convenient types that I reuse here
use audio_visualizer::{ChannelInterleavement, Channels};
use lowpass_filter::simple::sp::apply_lpf_i16_sp;

/// Minimal example how to use this crate/how to apply low pass filter.
fn main() {
    // read this from MP3 for example
    let audio_data_lrlr = [0_i16, 1, -5, 1551, 141, 24];

    // split into left and right channel
    // audio_visualizer has some cool convenient types that I reuse here
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
```

## Visual Examples
### #1: Original Waveform of a short sample
![Example 1: Original Waveform of a short sample](sample1_waveform.png "Example 1: Original Waveform of a short sample")
### #1: Lowpassed Waveform
![Example 1: Lowpassed Waveform of a short sample](sample1_waveform_lowpassed.png "Example 1: Lowpassed Original Waveform of a short sample")
### #2: Original Waveform of a song
![Example 1: Original Waveform of a song](song_waveform.png "Example 1: Original Waveform of a song")
### #2: Lowpassed Waveform
![Example 1: Lowpassed Waveform of a song](song_waveform_lowpassed.png "Example 1: Lowpassed Original Waveform of a song")
### #2: 3x Lowpassed Waveform
![Example 1: Lowpassed Waveform of a song 3x](song_waveform_lowpassed_3x.png "Example 1: Lowpassed Original Waveform of a song 3 times")
