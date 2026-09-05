# Changelog for `lowpass-filter`

## Unreleased
- Significantly improved performance. Compared to the previous release,
  expect roughly 1.5x throughput from the existing iterator-based API and
  roughly 3-5x from the new slice-based API (measured on x86_64; exact
  factors vary with CPU and compiler flags).
- Added `LowpassFilter::run_slice`, `lowpass_filter_slice`, and
  `lowpass_filter_slice_f64`: filter whole slices with a block-based
  algorithm that compilers can auto-vectorize (SIMD). Results match the
  per-sample API up to tiny floating point rounding differences
  (roughly `1e-6` for `f32`).

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
