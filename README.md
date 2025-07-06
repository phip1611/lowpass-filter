# lowpass-filter

Simple first-order digital lowpass filters, compatible with `no_std`. You can
use it, for example, to get the low frequencies from a song.

## Difference to `biquad`

**⚠ TL;DR: `biquad` might be a better option in some use-cases.**

This crate provides a basic and simple to understand, first order lowpass
filter. The [biquad](https://crates.io/crates/biquad) crate offers second order
filters, with higher accuracy. From my testing, a lowpass filter created with
`biquad` has higher computational costs as this crate, but offers a
**better resolution for actually cutting of signals above the cut-off frequency
while the preserved signal will be less attenuated**.

So for production use-cases, please also consider using `biquad`. You can run
benchmark and check your data (e.g., by plotting) to make that decision.

## How to use
```rust
use lowpass_filter::lowpass_filter;

/// Minimal example how to use this crate/how to apply low pass filter.
fn main() {
    // read this from MP3 for example
    let mut mono_audio_data = [0.0, 1.0, -5.0, 1551.0, 141.0, 24.0];
    // mutates the input buffer
    lowpass_filter(&mut mono_audio_data, 44100.0, 120.0);
}
```

## Visual Examples
### #1: Original Waveform of a short sample
![Example 1: Original Waveform of a short sample](res/sample1_waveform.png "Example 1: Original Waveform of a short sample")
### #1: Lowpassed Waveform
![Example 1: Lowpassed Waveform of a short sample](res/sample1_waveform_lowpassed.png "Example 1: Lowpassed Original Waveform of a short sample")
### #2: Original Waveform of a song
![Example 1: Original Waveform of a song](res/song_waveform.png "Example 1: Original Waveform of a song")
### #2: Lowpassed Waveform
![Example 1: Lowpassed Waveform of a song](res/song_waveform_lowpassed.png "Example 1: Lowpassed Original Waveform of a song")
### #2: 3x Lowpassed Waveform
![Example 1: Lowpassed Waveform of a song 3x](res/song_waveform_lowpassed_3x.png "Example 1: Lowpassed Original Waveform of a song 3 times")

# MSRV
The MSRV is `1.85.0`.
