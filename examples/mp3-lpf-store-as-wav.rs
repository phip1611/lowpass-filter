use std::fs::File;
use std::path::{Path, PathBuf};

use audio_visualizer::spectrum::plotters_png_file::spectrum_static_plotters_png_visualize;
use audio_visualizer::waveform::png_file::waveform_static_png_visualize;
use audio_visualizer::{ChannelInterleavement, Channels};
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};
use lowpass_filter::lowpass_filter;
use minimp3::{Decoder as Mp3Decoder, Error as Mp3Error, Frame as Mp3Frame};
use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use wav::{BitDepth, Header};

/// Takes a path to an mp3 as first argument,
/// applies a low pass filter n times (second argument)
/// and stores the file with suffix "_lowpassed" as wav file
/// (because yet there is no mp3 encoding crate).
fn main() {
    let env = std::env::args().collect::<Vec<String>>();
    let path = env.get(1).map(PathBuf::from).expect("Must provide path!");
    let times = env
        .get(2)
        .map(|s| s.parse().expect("Must be valid number"))
        .unwrap_or(1);

    // READ MP3 and split into left and right channel
    let (lrlr_mp3_samples, mp3_sample_rate) = mp3_to_lrlr_audio(path.as_path());
    let (left, right) = Channels::Stereo(ChannelInterleavement::LRLR)
        .stereo_interleavement()
        .to_channel_data(&lrlr_mp3_samples);

    // TOOO lets of code repetition.. could be much nicer..

    // lowpass-filter function of this crate
    {
        let mut lpf_left = samples_to_f32(&left);
        let mut lpf_right = samples_to_f32(&right);

        // STORE SPECTRUM AS FILE BEFORE LPF
        samples_to_spectrum_and_plot(&lpf_left, mp3_sample_rate, "mp3-original-spectrum.png");
        waveform_static_png_visualize(
            &left,
            Channels::Mono,
            "test/out",
            "mp3-original-waveform--left.png",
        );
        waveform_static_png_visualize(
            &right,
            Channels::Mono,
            "test/out",
            "mp3-original-waveform--right.png",
        );

        // APPLY LPF n TIMES
        for _ in 0..times {
            // left
            lowpass_filter(&mut lpf_left, mp3_sample_rate, 80.0);
            // right
            lowpass_filter(&mut lpf_right, mp3_sample_rate, 80.0);
        }

        // STORE DATA AS WAV
        store_data_as_wav(
            &samples_to_i16(&lpf_left),
            &samples_to_i16(&lpf_right),
            path.as_path(),
            mp3_sample_rate,
            "--lowpassed.wav",
        );

        // STORE SPECTRUM AS FILE AFTER LPF
        samples_to_spectrum_and_plot(&lpf_left, mp3_sample_rate, "mp3-lowpassed-spectrum.png");
        waveform_static_png_visualize(
            &lpf_left.iter().map(|x| *x as i16).collect::<Vec<_>>(),
            Channels::Mono,
            "test/out",
            "mp3-lowpassed-waveform--left.png",
        );
        waveform_static_png_visualize(
            &lpf_right.iter().map(|x| *x as i16).collect::<Vec<_>>(),
            Channels::Mono,
            "test/out",
            "mp3-lowpassed-waveform--right.png",
        );
    }
    // for comparison: biquad lowpass filter
    {
        // Cutoff and sampling frequencies

        let mut left_biquad_lpf = samples_to_f32(&left);
        let mut right_biquad_lpf = samples_to_f32(&right);

        // APPLY LPF n TIMES
        for _ in 0..times {
            left_biquad_lpf = apply_biquad_lowpass_filter(&left_biquad_lpf, mp3_sample_rate, 80.0);
            right_biquad_lpf =
                apply_biquad_lowpass_filter(&right_biquad_lpf, mp3_sample_rate, 80.0);
        }

        // STORE DATA AS WAV
        store_data_as_wav(
            &samples_to_i16(&left_biquad_lpf),
            &samples_to_i16(&right_biquad_lpf),
            path.as_path(),
            mp3_sample_rate,
            "--biquad-lowpassed.wav",
        );

        // STORE SPECTRUM AS FILE AFTER LPF
        samples_to_spectrum_and_plot(
            &left_biquad_lpf,
            mp3_sample_rate,
            "mp3-biquad-lowpassed-spectrum.png",
        );
        waveform_static_png_visualize(
            &samples_to_i16(&left_biquad_lpf),
            Channels::Mono,
            "test/out",
            "mp3-biquad-lowpassed-waveform--left.png",
        );
        waveform_static_png_visualize(
            &samples_to_i16(&right_biquad_lpf),
            Channels::Mono,
            "test/out",
            "mp3-biquad-lowpassed-waveform--right.png",
        );
    }
}

/// Reads the mp3 to a vector of the decoded LRLR (left, right, left, right)
/// data. Second return value is the sampling rate.
fn mp3_to_lrlr_audio(path: &Path) -> (Vec<i16>, f32) {
    let mut decoder = Mp3Decoder::new(File::open(&path).unwrap());

    let mut lrlr_mp3_samples = vec![];
    let mut mp3_sample_rate = None;
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame {
                data: samples_of_frame,
                sample_rate,
                ..
            }) => {
                if mp3_sample_rate.is_none() {
                    mp3_sample_rate.replace(sample_rate as f32);
                }
                for sample in samples_of_frame {
                    lrlr_mp3_samples.push(sample);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    (lrlr_mp3_samples, mp3_sample_rate.unwrap())
}

fn samples_to_spectrum_and_plot(audio_data: &[f32], sampling_rate: f32, filename: &str) {
    let exponent = (audio_data.len() as f32).log2().floor();
    let fft_len = 2.0_f32.powf(exponent) as usize;
    let samples_for_spectrum = hann_window(&audio_data[0..fft_len]);
    let original_spectrum = samples_fft_to_spectrum(
        &samples_for_spectrum,
        sampling_rate as u32,
        FrequencyLimit::Max(5000.0),
        Some(&scale_to_zero_to_one),
    )
    .unwrap();
    spectrum_static_plotters_png_visualize(&original_spectrum.to_map(None), "test/out", filename);
}

fn store_data_as_wav(
    left_audio: &[i16],
    right_audio: &[i16],
    path: &Path,
    sample_rate: f32,
    extension: &str,
) {
    let mut stereo_lrlr_data = Vec::with_capacity(left_audio.len() * 2);
    for i in 0..left_audio.len() {
        stereo_lrlr_data.push(left_audio[i]);
        stereo_lrlr_data.push(right_audio[i]);
    }

    let original_filename = path.file_name().unwrap().to_str().unwrap();
    let new_path = path.with_file_name(&format!("{}{}", original_filename, extension));
    let mut out_file = File::create(Path::new(&new_path)).unwrap();
    wav::write(
        Header::new(0x01, 2, sample_rate as u32, 16),
        &BitDepth::Sixteen(stereo_lrlr_data),
        &mut out_file,
    )
    .unwrap();
}

// used to compare my lowpass filter (first order) against a second order biquad filter
fn apply_biquad_lowpass_filter(audio: &[f32], sampling_rate: f32, cutoff_fr: f32) -> Vec<f32> {
    let f0 = cutoff_fr.hz();
    let fs = sampling_rate.hz();

    // Create coefficients for the biquads
    let coeffs =
        Coefficients::<f32>::from_params(Type::BandPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
    let mut lowpassed_data = Vec::with_capacity(audio.len());
    let mut biquad_lpf = DirectForm1::<f32>::new(coeffs);
    audio
        .iter()
        .for_each(|val| lowpassed_data.push(biquad_lpf.run(*val)));
    lowpassed_data
}

fn samples_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|x| *x * (i16::MAX as f32))
        .map(|x| x as i16)
        .collect()
}

fn samples_to_f32(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|x| (*x as f32) / (i16::MAX as f32))
        .collect()
}
