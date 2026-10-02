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

## Performance

**TL;DR:** Prefer `run_slice`: on x86, it is 2-4x faster than per-sample
processing. The optional `simd` feature makes `run_slice()` even faster: up to
1.5x on x86 and 2.3x on ARM. For streaming, create the filter once and reuse
it.

Measured with Rust 1.99, filtering one second of audio (44100 samples) with a
fresh filter.

- _Default_: compiled for the baseline CPU of the target. On `x86_64`, this
  only guarantees SSE2 with 128-bit SIMD registers. This is how most crates
  and binaries are built.
- _Native_: compiled with `-C target-cpu=native`, which enables all CPU
  features of the test machine, including AVX-512 with 512-bit SIMD
  registers on both x86 CPUs. Such binaries do not run on CPUs without these
  features. On the Raspberry Pi, it makes no difference.

Throughput of `f32` samples in million samples per second, higher is better:

| CPU                               | `run` | `run_slice` | `run_slice` (`simd`) | `biquad` |
|-----------------------------------|------:|------------:|---------------------:|---------:|
| AMD EPYC 9634, default            |   603 |        1503 |                 2192 |      394 |
| AMD EPYC 9634, native             |   596 |        2478 |                 3637 |      401 |
| AMD Ryzen 7 7840U, default        |   828 |        2052 |                 3051 |      546 |
| AMD Ryzen 7 7840U, native         |   823 |        3160 |                 4893 |      548 |
| Raspberry Pi 4 (Cortex-A72)       |   187 |         215 |                  492 |      143 |

With `f64` samples, `run` and `biquad` perform the same, while `run_slice`
reaches 55-90% of the `f32` throughput.

When processing audio in buffers of 64 samples, reusing one filter is
1.1-2.2x faster than creating a new one per buffer, and reaches at least 90%
of the throughput of filtering one long slice.

### Explicit SIMD

The optional `simd` cargo feature makes `run_slice` use explicit SIMD via the
[wide](https://crates.io/crates/wide) crate instead of relying on compiler
auto-vectorization. It is faster on all measured CPUs, most for `f32`. The
trade-off is a dependency and an MSRV of Rust 1.89. Without the feature, this
crate has no dependencies.

```toml
lowpass-filter = { version = "0.6", features = ["simd"] }
```

### Comparison with `biquad`

For the equivalent first-order lowpass (`Type::SinglePoleLowPass`), this
crate outperforms the [biquad](https://crates.io/crates/biquad) crate:
1.3-1.5x throughput with per-sample processing (`run`) and 1.4-6.2x with
slice processing (`run_slice`), depending on the CPU, the sample type, and
the enabled CPU features, as `biquad` processes samples strictly one at a
time. With the `simd` feature, it is up to 9x.

`biquad` is the better choice for sharp frequency separation or other
filter types (highpass, bandpass, notch, EQ): its second-order filters
roll off at 12 dB/octave instead of 6, with a configurable Q factor.

### Running the Benchmarks

```sh
# Baseline CPU features
cargo bench -- --save-baseline generic
# All CPU features of this machine, compared against the run above
RUSTFLAGS="-C target-cpu=native" cargo bench -- --baseline generic
# With the simd feature, compared against the run above
cargo bench --features simd -- --baseline generic
# Quick and rough, without touching saved results
cargo bench -- --quick --noplot --discard-baseline
```

Pass a filter to run a subset, e.g. `cargo bench -- streaming/f32`. The HTML
report is written to `target/criterion/report/index.html`.

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
