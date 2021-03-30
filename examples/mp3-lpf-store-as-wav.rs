use std::fs::File;
use lowpass_filter::simple::sp::apply_lpf_i16_sp;
use minimp3::{Decoder as Mp3Decoder, Error as Mp3Error, Frame as Mp3Frame};
use std::path::{Path, PathBuf};
use wav::{Header, BitDepth};
use spectrum_analyzer::{FrequencyLimit, SpectrumTotalScaleFunctionFactory};
use audio_visualizer::spectrum::staticc::plotters_png_file::spectrum_static_plotters_png_visualize;
use audio_visualizer::test_support::TEST_OUT_DIR;
use spectrum_analyzer::windows::hann_window;
use audio_visualizer::{Channels, ChannelInterleavement};

/// Takes a path to an mp3 as first argument,
/// applies a low pass filter n times (second argument)
/// and stores the file with suffix "_lowpassed" as wav file
/// (because yet there is no mp3 encoding crate).
fn main() {
    let env = std::env::args().collect::<Vec<String>>();
    let path = env.get(1).map(|p| PathBuf::from(p)).expect("Must provide path!");
    let times = env.get(2).map(|s| s.parse().expect("Must be valid number")).unwrap_or(1);

    let mut decoder = Mp3Decoder::new(File::open(&path).unwrap());

    let mut lrlr_mp3_samples = vec![];
    let mut mp3_sample_rate: u32 = 0;
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame {
                   data: samples_of_frame,
                   sample_rate,
                  ..
               }) => {
                mp3_sample_rate = sample_rate as u32;
                for sample in samples_of_frame {
                    lrlr_mp3_samples.push(sample);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    let (mut left, mut right) = Channels::Stereo(ChannelInterleavement::LRLR)
        .stereo_interleavement()
        .to_channel_data(&lrlr_mp3_samples);

    // get next lower base of 2
    let fft_analyze_len = 2_u32.pow((left.len() as f32).log2() as u32) as usize;

    let samples_for_spectrum = left.iter().take(fft_analyze_len).map(|i| *i as f32).collect::<Vec<f32>>();
    let samples_for_spectrum = hann_window(&samples_for_spectrum);
    let original_spectrum = spectrum_analyzer::samples_fft_to_spectrum(
        &samples_for_spectrum,
        mp3_sample_rate,
        FrequencyLimit::Max(10000.0),
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    spectrum_static_plotters_png_visualize(
        &original_spectrum.to_map(None),
        TEST_OUT_DIR,
        "mp3-original-spectrum.png",
    );

    for _ in 0..times {
        // left
        apply_lpf_i16_sp(&mut left, 44100, 120);
        // right
        apply_lpf_i16_sp(&mut right, 44100, 120);
    }

    let samples_for_spectrum = left.iter().take(fft_analyze_len).map(|i| *i as f32).collect::<Vec<f32>>();
    let samples_for_spectrum = hann_window(&samples_for_spectrum);
    let original_spectrum = spectrum_analyzer::samples_fft_to_spectrum(
        &samples_for_spectrum,
        mp3_sample_rate,
        FrequencyLimit::Max(10000.0),
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    spectrum_static_plotters_png_visualize(
        &original_spectrum.to_map(None),
        TEST_OUT_DIR,
        "mp3-original-spectrum--lowpassed.png",
    );

    let mut stereo_lrlr_data = Vec::with_capacity(left.len() * 2);
    for i in 0..left.len() {
        stereo_lrlr_data.push(left[i]);
        stereo_lrlr_data.push(right[i]);
    }

    let original_filename = path.file_name().unwrap().to_str().unwrap();
    let new_path = path.with_file_name(&format!("{}--lowpassed.wav", original_filename));
    let mut out_file = File::create(Path::new(&new_path)).unwrap();
    wav::write(
        Header::new(
            0x01,
            2,
            mp3_sample_rate,
            16,
        ),
        &BitDepth::Sixteen(stereo_lrlr_data),
        &mut out_file
    ).unwrap();
}

fn get_scale_to_one_fn_factory() -> SpectrumTotalScaleFunctionFactory {
    Box::new(move |_min: f32, max: f32, _average: f32, _median: f32| Box::new(move |x| x / max))
}