use audio_visualizer::dynamic::live_input::AudioDevAndCfg;
use audio_visualizer::dynamic::window_top_btm::{TransformFn, open_window_connect_audio};
use lowpass_filter::lowpass_filter;

/// Example that creates a live visualization of realtime audio data
/// through a lowpass filter. **Execute this with `--release`, otherwise it is very laggy!**.
fn main() {
    open_window_connect_audio(
        "Live Audio Lowpass Filter View",
        None,
        None,
        None,
        None,
        "time (seconds)",
        "Amplitude (with Lowpass filter)",
        // fall back to the default input audio device (e.g. microphone)
        AudioDevAndCfg::new(None, None),
        // lowpass filter
        TransformFn::Basic(|x, sampling_rate| {
            let mut data = x.to_vec();
            lowpass_filter(&mut data, sampling_rate, 120.0);
            data
        }),
    );
}
