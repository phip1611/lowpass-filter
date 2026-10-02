use crate::test_util::{
    f32_sample_to_i16, i16_sample_to_f32, read_wav_to_mono, target_dir_test_artifacts,
    write_wav_file,
};
use audio_visualizer::WaveformVisualizer;
use lowpass_filter::LowpassFilter;
use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
use std::path::PathBuf;

#[path = "../src/test_util.rs"]
mod test_util;

/// CLI utility that takes two arguments:
/// - a path to a wav file
/// - a number that specifies the amount of lowpass filter iterations
///
/// It will then store a new wav file (mono channel and lowpassed) next to
/// the original file.
fn main() {
    let env = std::env::args().collect::<Vec<String>>();
    let path = env
        .get(1)
        .map(PathBuf::from)
        .expect("Must provide path as first argument!");
    let times = env
        .get(2)
        .map(|s| {
            s.parse()
                .expect("Number of lowpass filter iterations must be valid number")
        })
        .unwrap_or(1);

    let (samples_unprocessed_i16, wavspec) = read_wav_to_mono(&path);
    let samples_unprocessed_f32 = samples_unprocessed_i16
        .iter()
        .copied()
        .map(i16_sample_to_f32)
        .collect::<Vec<_>>();

    // Store plotted spectrum before any processing
    samples_to_spectrum_and_plot(
        &samples_unprocessed_f32[0..16384],
        wavspec.sample_rate as f32,
        "wav-original-spectrum--mono.png",
    );

    WaveformVisualizer::new(&samples_unprocessed_f32)
        .sample_rate(wavspec.sample_rate as f32)
        .y_range(-1.0..1.0)
        .write_png(format!(
            "{}/wav-original-waveform--mono.png",
            target_dir_test_artifacts().display()
        ))
        .unwrap();

    // Apply LPF n times
    let mut samples_processed_f32 = samples_unprocessed_f32.clone();
    for _ in 0..times {
        LowpassFilter::new(wavspec.sample_rate as f32, 100.0).run_slice(&mut samples_processed_f32);
    }
    let samples_processed_i16 = samples_processed_f32
        .iter()
        .copied()
        .map(f32_sample_to_i16)
        .collect::<Vec<_>>();

    // add suffix to path
    let new_wav_path = {
        let parent = path.parent().unwrap();
        let stem = path
            .file_stem()
            .expect("Path has no file name")
            .to_str()
            .unwrap();
        let new_filename = format!("{stem}_processed.wav");
        parent.join(new_filename)
    };

    // STORE DATA AS WAV
    write_wav_file(&new_wav_path, &samples_processed_i16, wavspec.sample_rate);

    // STORE SPECTRUM AS FILE AFTER LPF
    samples_to_spectrum_and_plot(
        &samples_processed_f32[0..16384],
        wavspec.sample_rate as f32,
        "wav-lowpassed-spectrum--mono.png",
    );
    WaveformVisualizer::new(&samples_processed_f32)
        .sample_rate(wavspec.sample_rate as f32)
        .y_range(-1.0..1.0)
        .write_png(format!(
            "{}/wav-lowpassed-waveform--mono.png",
            target_dir_test_artifacts().display()
        ))
        .unwrap();
}

fn samples_to_spectrum_and_plot(audio_data: &[f32], sampling_rate: f32, filename: &str) {
    let exponent = (audio_data.len() as f32).log2().floor();
    let fft_len = 2.0_f32.powf(exponent) as usize;
    let samples_for_spectrum = hann_window(&audio_data[0..fft_len]);
    let spectrum = samples_fft_to_spectrum(
        &samples_for_spectrum,
        sampling_rate as u32,
        FrequencyLimit::max(5000.0),
        Some(&scale_to_zero_to_one),
    )
    .unwrap();

    let spectrum_raw = spectrum
        .data()
        .iter()
        .map(|(a, b)| (a.val(), b.val()))
        .collect::<Vec<_>>();

    audio_visualizer::SpectrumVisualizer::new(&spectrum_raw[0..16384])
        .write_png(format!(
            "{}/{filename}",
            target_dir_test_artifacts().display()
        ))
        .unwrap();
}
