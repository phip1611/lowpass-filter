# lowpass-filter

High performance `no_std` lowpass filter for digital signal processing.

This crate implements a simple first-order digital lowpass filter for `f32`
and `f64` samples. Use it, for example, to extract the bass from a song or to
smooth noisy sensor data. It has no dependencies, no `unsafe` code, and
performs no allocations, making it suitable for any target from desktop to
embedded.

Samples should be in range `-1.0..=1.0`, which is the default in DSP.

## Usage

```rust
use lowpass_filter::LowpassFilter;

// Mono audio samples, recorded at 44.1 kHz sample rate.
let mut samples = [0.0_f32, 0.3, -0.6, 0.8, 0.5, -0.2];
// Only keep frequencies below 120 Hz; mutates the buffer in-place.
LowpassFilter::new(44100.0, 120.0).run_slice(&mut samples);
```

For streaming, per-sample processing, and more examples, see the
[documentation](https://docs.rs/lowpass-filter).

## Comparison with `biquad`

For the equivalent first-order lowpass (`Type::SinglePoleLowPass`), this
crate outperforms the [biquad](https://crates.io/crates/biquad) crate:
roughly 1.5x throughput with per-sample processing (`run`) and 5-6x with
slice processing (`run_slice`) on x86_64, as `biquad` processes samples
strictly one at a time.

`biquad` is the better choice for sharp frequency separation or other
filter types (highpass, bandpass, notch, EQ): its second-order filters
roll off at 12 dB/octave instead of 6, with a configurable Q factor.

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
The MSRV of the library is `1.88.0`. The MSRV of the benches, examples, and
tests in this repository is `1.95.0`.
