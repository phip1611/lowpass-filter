# Changelog for `lowpass-filter`

## Unreleased

## v0.6.0 (2026-10-02)
- Breaking: Removed `lowpass_filter`, `lowpass_filter_f64`,
  `lowpass_filter_slice`, and `lowpass_filter_slice_f64`. `LowpassFilter` is
  now the only entry point:
  - `lowpass_filter_slice(&mut s, sr, fc)` ->
    `LowpassFilter::new(sr, fc).run_slice(&mut s)`
  - `lowpass_filter(&mut s, sr, fc)` -> same as above; for samples not in a
    slice, call `LowpassFilter::run` per sample.
- `LowpassFilter` now defaults to `f32` samples, e.g.,
  `struct S { filter: LowpassFilter }`
- Samples are no longer clamped to `-1.0..=1.0`, but this range is still highly
  recommended.
- Added the optional `simd` cargo feature: `LowpassFilter::run_slice` then uses
  explicit SIMD via the `wide` crate. It requires Rust 1.89; without it, this
  crate stays dependency-free.
- Performance:
  - Without clamping, `run` is about 8% and `run_slice` up to 50% faster.
  - `run_slice` is about 5% faster when called with fewer than 512 samples, as
    in streaming.
  - `run_slice` processes blocks of 4 samples instead of 8, unless AVX2 is
    enabled at compile time (e.g., `-C target-cpu=x86-64-v3`). This is up to
    2x faster for short inputs, streaming, and `f64`. Without the `simd`
    feature, `f32` calls with 1024 or more samples can be up to 18% slower.
  - The `simd` feature makes `run_slice` about 1.5x faster for `f32` on x86,
    also with the default target, and 2.3x on aarch64.
  - Compiling for `x86-64-v3` or newer (e.g., `-C target-cpu=native`) makes
    `run_slice` faster, with and without the `simd` feature.

## v0.5.0 (2026-09-05)
- Significantly improved performance. Compared to the previous release,
  expect roughly 1.5x throughput from the existing iterator-based API and
  roughly 3-5x from the new slice-based API (measured on x86_64; exact
  factors vary with CPU and compiler flags).
- Added `LowpassFilter::run_slice`, `lowpass_filter_slice`, and
  `lowpass_filter_slice_f64`: filter whole slices with a block-based
  algorithm that compilers can auto-vectorize (SIMD). Results match the
  per-sample API up to tiny floating point rounding differences
  (roughly `1e-6` for `f32`).
- `LowpassFilter` methods are now a generic implementation over the new
  sealed `Sample` trait (implemented for `f32` and `f64`) instead of
  macro-generated per-type implementations. This also improved slice
  throughput further, as monomorphization now happens in the consuming
  crate with its target flags.
- MSRV is now 1.88.0 (required by `slice::as_chunks_mut`)

## v0.4.1 (2025-07-06)
- doc updates

## v0.4.0 (2025-07-06)
- modernized crate, Rust edition 2024
- Added new `LowpassFilter` type that makes it easier to use this crate
- MSRV is now 1.85.0

## v0.3.1/0.3.2 (2021-11-15)
- smaller crate size/don't include irrelevant stuff

## v0.3.0 (2021-11-15)
- MSRV is 1.56.1
- crate uses Rust edition 2021
- improved and simplified lib structure
- library function has more sensible input type (f32)
