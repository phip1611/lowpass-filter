//! Simple lowpass filter.

/// Applies a low pass filter on a vector of **mono sample**.
fn apply_lowpass(data: &mut [i16], sample_rate_hz: u16, cutoff_frequency_hz: u16) {
    let rc = 1_f64 / (cutoff_frequency_hz as f64 * 2_f64 * std::f64::consts::PI);
    let dt = 1_f64 / sample_rate_hz as f64;
    let alpha = dt / (rc + dt);

    let mut cpy_orig_data = Vec::new();
    cpy_orig_data.extend_from_slice(data);

    data[0] = (alpha * cpy_orig_data[0] as f64) as i16;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        data[i] = (data[i - 1] as f64
            + alpha *
            (cpy_orig_data[i] as f64 - data[i-1] as f64)) as i16;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
    use crate::test::{TEST_SAMPLES_DIR, TEST_OUT_DIR};
    use std::path::PathBuf;
    use std::fs::File;
    use audio_visualizer::{Channels, ChannelInterleavement};
    use audio_visualizer::waveform::staticc::png_file::visualize;
    use std::time::Instant;

    #[test]
    fn test_visualize_1() {
        let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
        path.push("sample_1.mp3");
        let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

        let mut lrlr_mp3_samples = vec![];
        loop {
            match decoder.next_frame() {
                Ok(Mp3Frame { data: samples_of_frame, .. }) => {
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


        let now = Instant::now();
        // left
        apply_lowpass(&mut left, 44100, 150);
        let then = now.elapsed();
        println!("took {}us", then.as_micros());
        // right
        apply_lowpass(&mut right, 44100, 150);

        visualize(
            &left,
            Channels::Mono,
            TEST_OUT_DIR,
            "sample_1_waveform_lowpassed_left.png"
        );
        visualize(
            &right,
            Channels::Mono,
            TEST_OUT_DIR,
            "sample_1_waveform_lowpassed_right.png"
        );
    }
}
