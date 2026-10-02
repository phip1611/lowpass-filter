/*
MIT License

Copyright (c) 2026 Philipp Schuster

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
//! Explicit SIMD implementation of [`LowpassFilter::run_slice`] using the
//! [`wide`] crate.

use crate::util::RunSliceSimd;
use crate::{LANES, LowpassFilter, Sample};
use wide::{f32x8, f64x8};

/// Explicit SIMD implementation of [`LowpassFilter::run_slice`].
#[inline]
pub fn run_slice_simd<T: Sample>(filter: &mut LowpassFilter<T>, samples: &mut [T]) {
    T::run_slice_simd(filter, samples);
}

macro_rules! impl_run_slice_simd {
    ($t:ty, $v:ty) => {
        impl RunSliceSimd for $t {
            #[inline]
            fn run_slice_simd(filter: &mut LowpassFilter<Self>, samples: &mut [Self]) {
                let weights = filter.weights.map(<$v>::new);
                let carry_coeffs = <$v>::new(filter.carry_coeffs);
                let carry_last = filter.carry_coeffs[LANES - 1];

                let (chunks, remainder) = samples.as_chunks_mut::<LANES>();
                let mut prev = filter.prev;
                for chunk in chunks {
                    // Each output consists of two parts: one from this block's
                    // input samples and one from the last output of the
                    // previous block (`prev`). First, add up the part from the
                    // input samples. It does not need the previous block, so
                    // the CPU can already work on it while the previous block
                    // is still being finished.
                    let mut acc = <$v>::splat(0.0);
                    for (weight, &sample) in weights.iter().zip(chunk.iter()) {
                        acc = weight.mul_add(<$v>::splat(sample), acc);
                    }
                    // Then add the part from the previous block.
                    let out = carry_coeffs.mul_add(<$v>::splat(prev), acc);

                    // The next block needs this block's last output as `prev`.
                    // This hand-over is the only part that cannot overlap
                    // between blocks: thanks to out-of-order execution, the CPU
                    // can already add up the input samples of the next blocks,
                    // but it cannot finish a block before its `prev` is known.
                    // Keep the hand-over short, so that the CPU can keep more
                    // blocks in flight: compute the last output a second time
                    // as a plain number (scalar), instead of taking it out of
                    // the vector `out` and spreading it into a vector again for
                    // the next block. For f32, this is up to 37% faster than
                    // `prev = out.as_array()[LANES - 1]`.
                    prev = carry_last * prev + acc.as_array()[LANES - 1];
                    *chunk = out.to_array();
                }
                filter.prev = prev;

                for sample in remainder {
                    *sample = filter.run(*sample);
                }
            }
        }
    };
}

impl_run_slice_simd!(f32, f32x8);
impl_run_slice_simd!(f64, f64x8);
