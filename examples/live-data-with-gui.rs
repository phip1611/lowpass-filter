use audio_visualizer::live::{AudioInput, LiveVisualizer, Transform};
use lowpass_filter::LowpassFilter;
use std::io::{BufRead, stdin};

/// Example that creates a live visualization of realtime audio data
/// through a lowpass filter. **Execute this with `--release`, otherwise it is very laggy!**.
fn main() {
    let input = select_input();
    LiveVisualizer::new(Transform::waveform(|samples, sample_rate| {
        // `samples` is an overlapping history window, not a stream of new
        // samples, so filter it as a whole instead of keeping a filter
        // across calls.
        let mut samples = samples.to_vec();
        LowpassFilter::new(sample_rate, 80.0).run_slice(&mut samples);
        samples
    }))
    .title("Live Audio Lowpass Filter View")
    .axis_labels("time (seconds)", "amplitude (lowpass filtered)")
    .input(input)
    .open()
    .unwrap();
}

/// Lets the user select an audio input device on stdin if there are multiple.
fn select_input() -> AudioInput {
    let mut devs = AudioInput::devices().unwrap();
    assert!(!devs.is_empty(), "no audio input devices found!");
    if devs.len() == 1 {
        return AudioInput::from_device(devs.remove(0).1).unwrap();
    }
    println!();
    devs.iter()
        .enumerate()
        // skip devices without a usable input config, and show the config
        // that would actually be recorded with
        .filter_map(|(i, (name, dev))| {
            AudioInput::from_device(dev.clone())
                .ok()
                .map(|input| (i, name, input))
        })
        .for_each(|(i, name, input)| {
            println!("  [{i}] {name} {:?}", input.config());
        });
    let mut input = String::new();
    stdin().lock().read_line(&mut input).unwrap();
    let index = input.trim().parse::<usize>().unwrap();
    AudioInput::from_device(devs.remove(index).1).unwrap()
}
