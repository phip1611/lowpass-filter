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
//! Simple first order low pass filter as described on
//! [Wikipedia](https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter).
//!
//! It applies a low pass filter on a vector of samples. It mutates the input array.
//! Therefore, the number of output values equals the number of input values.

use alloc::vec::Vec;
use core::f32;
use core::f64;
use core::ops::{Add, Div, Mul, Sub};

trait FloatTrait:
    Mul<Output = Self> + Div<Output = Self> + Add<Output = Self> + Sub<Output = Self> + Sized + Copy
{
    fn pi() -> Self;
    fn one() -> Self;
    fn two() -> Self;
}

impl FloatTrait for f32 {
    fn pi() -> Self {
        f32::consts::PI
    }

    fn one() -> Self {
        1.0
    }

    fn two() -> Self {
        2.0
    }
}
impl FloatTrait for f64 {
    fn pi() -> Self {
        f64::consts::PI
    }

    fn one() -> Self {
        1.0
    }

    fn two() -> Self {
        2.0
    }
}

trait FirstOrderLowPassFilterTrait<FloatType, SampleType, SamplingRateType, CutoffFrType>
where
    FloatType: FloatTrait,
    SampleType: Into<FloatType> + Copy,
    SamplingRateType: Into<FloatType>,
    CutoffFrType: Into<FloatType>,
{
    fn apply(
        samples: &[SampleType],
        sampling_rate: SamplingRateType,
        cutoff_frequency_hz: CutoffFrType,
    ) -> Vec<FloatType> {
        let mut lp_samples = Vec::with_capacity(samples.len());
        let sampling_rate: FloatType = sampling_rate.into();
        let cutoff_frequency_hz: FloatType = cutoff_frequency_hz.into();
        let rc = FloatType::one() / (cutoff_frequency_hz * FloatType::two() * FloatType::pi());
        let dt = FloatType::one() / sampling_rate;
        let alpha = dt / (rc + dt);

        lp_samples[0] = alpha * samples[0].into();
        for i in 1..samples.len() {
            // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter

            // the original data is accessed before it is overwritten:
            // data[i] = ... data[i]

            lp_samples[i] = lp_samples[i - 1] + alpha * (samples[i].into() - lp_samples[i - 1]);
        }

        lp_samples
    }
}

struct Filter;
impl<FloatType, SampleType, SamplingRateType, CutoffFrType>
    FirstOrderLowPassFilterTrait<FloatType, SampleType, SamplingRateType, CutoffFrType> for Filter
where
    FloatType: FloatTrait,
    SampleType: Into<FloatType> + Copy,
    SamplingRateType: Into<FloatType>,
    CutoffFrType: Into<FloatType>,
{
}

#[cfg(test)]
mod tests2 {

    use super::*;

    #[test]
    fn test_fo_lw_filter() {
        let samples_i32 = [
            2, -2, 2, -2, 4, -4, 6, -6
        ];
        let samples_f32 = [
            2.0_f32, -2.0, 2.0, -2.0, 4.0, -4.0, 6.0, -6.0
        ];
        let samples_f64 = [
            2.0_f64, -2.0, 2.0, -2.0, 4.0, -4.0, 6.0, -6.0
        ];
        //let res: Vec<f32> = Filter::apply(&samples_f32, 44100, 120);
        let x: i16 = 5;
        let y: f32 = x.into();
    }
}

/// Common simple/first order low pass filter implementation for f32 and f64.
macro_rules! lpw_float_impl {
    ($samples: ident, $sampling_rate_hz: ident, $cutoff_frequency_hz: ident, $pi: path) => {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        let rc = 1.0 / ($cutoff_frequency_hz * 2.0 * $pi);
        let dt = 1.0 / $sampling_rate_hz;
        let alpha = dt / (rc + dt);

        $samples[0] = alpha * $samples[0];
        for i in 1..$samples.len() {
            // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter

            // we don't need a copy of the original data, because the original data is accessed
            // before it is overwritten: data[i] = ... data[i]

            $samples[i] = $samples[i - 1] + alpha * ($samples[i] - $samples[i - 1]);
        }
    };
}

/// Common trait used for various implementations on [`FirstOrderLowPassFilter`].
pub trait FirstOrderLowPassFilterFloatTrait<T> {
    fn filter(samples: &mut [T], sampling_rate_hz: T, cutoff_frequency_hz: T);
}

/// Zero-sized helper type.
/// Use the `filter` methods of the available implementations of [`FirstOrderLowPassFilterTrait`].
///
/// # Usage
/// ```ignore
/// LowPassFilter::filter::<f32::(samples, 44100.0, 120.0);
/// // or
/// LowPassFilter::filter::<64::(samples, 44100.0, 120.0);
/// ```
pub struct FirstOrderLowPassFilter;

impl FirstOrderLowPassFilterFloatTrait<f64> for FirstOrderLowPassFilter {
    /// Applies a first order low pass filter on an array of `f64` samples.
    /// It mutates the input array. Therefore, the number of output values
    /// equals the number of input values.
    fn filter(samples: &mut [f64], sampling_rate_hz: f64, cutoff_frequency_hz: f64) {
        lpw_float_impl!(
            samples,
            sampling_rate_hz,
            cutoff_frequency_hz,
            f64::consts::PI
        );
    }
}

impl FirstOrderLowPassFilterFloatTrait<f32> for FirstOrderLowPassFilter {
    /// Applies a first order low pass filter on an array of `f32` samples.
    /// It mutates the input array. Therefore, the number of output values
    /// equals the number of input values.
    fn filter(samples: &mut [f32], sampling_rate_hz: f32, cutoff_frequency_hz: f32) {
        lpw_float_impl!(
            samples,
            sampling_rate_hz,
            cutoff_frequency_hz,
            f32::consts::PI
        );
    }
}

#[cfg(test)]
mod tests {
    use super::FirstOrderLowPassFilter;
    use super::FirstOrderLowPassFilterFloatTrait;
    use crate::test_util::{TEST_OUT_DIR, TEST_SAMPLES_DIR};
    use alloc::vec::Vec;
    use audio_visualizer::waveform::staticc::png_file::waveform_static_png_visualize;
    use audio_visualizer::{ChannelInterleavement, Channels};
    use minimp3::{Decoder as Mp3Decoder, Error as Mp3Error, Frame as Mp3Frame};
    use std::fs::File;
    use std::path::PathBuf;
    use std::prelude::*;
    use std::time::Instant;

    /// To see if the test actually works, check the waveform in the image output.
    #[test]
    fn test_visualize_lowpassed_data() {
        let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
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
            TEST_OUT_DIR,
            "sample_1_waveform_original_before_lpf.png",
        );

        // split into left and right channel
        let (mut left, mut right) = Channels::Stereo(ChannelInterleavement::LRLR)
            .stereo_interleavement()
            .to_channel_data(&lrlr_mp3_samples);

        let (mut left, mut right) = (
            left.into_iter().map(|x| x as f32).collect::<Vec<_>>(),
            right.into_iter().map(|x| x as f32).collect::<Vec<_>>(),
        );

        let now = Instant::now();
        // left
        for _ in 0..3 {
            FirstOrderLowPassFilter::filter(&mut left, 44100.0, 120.0);
        }
        let then = now.elapsed();
        println!(
            "took {}us to apply low pass filter for left channel ({}) samples",
            then.as_micros(),
            left.len()
        );
        // right
        FirstOrderLowPassFilter::filter(&mut right, 44100.0, 120.0);

        let (left, right) = (
            left.into_iter().map(|x| x as i16).collect::<Vec<_>>(),
            right.into_iter().map(|x| x as i16).collect::<Vec<_>>(),
        );

        // visualize audio as waveform in a PNG file
        waveform_static_png_visualize(
            &left,
            Channels::Mono,
            TEST_OUT_DIR,
            "sample_1_waveform_lowpassed_3times_left.png",
        );
        waveform_static_png_visualize(
            &right,
            Channels::Mono,
            TEST_OUT_DIR,
            "sample_1_waveform_lowpassed_right.png",
        );
    }

    /// To see if the test actually works, check the waveform in the image output.
    #[test]
    fn test_compare_sp_dp_lpf() {
        /*let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
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


         */
    }
}
