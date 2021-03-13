//! Single precision / f32 / float.

use alloc::vec::Vec;

/// Applies a single precision (float, f32) low pass filter on a vector of **mono sample** in 16
/// bit resolution. If you have stereo data, call this function for each channel, convert it first
/// to mono or do whatever fits your use case.
///
/// ## Parameters
/// * `data` audio samples
/// * `sample_rate_hz` Sample Rate, e.g. 44100Hz
/// * `cutoff_frequency_hz` upper bound for frequencies to be cut, e.g. 150Hz
pub fn apply_lpf_i16_sp(data: &mut [i16], sample_rate_hz: u16, cutoff_frequency_hz: u16) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1.0 / (cutoff_frequency_hz as f32 * 2.0 * core::f32::consts::PI);
    let dt = 1.0 / sample_rate_hz as f32;
    let alpha = dt / (rc + dt);

    let mut cpy_orig_data = Vec::with_capacity(data.len());
    cpy_orig_data.extend_from_slice(data);

    data[0] = (alpha * cpy_orig_data[0] as f32) as i16;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        data[i] = (data[i - 1] as f32
            + alpha *
            (cpy_orig_data[i] as f32 - data[i-1] as f32)) as i16;
    }
}

/// Same as [`apply_lpf_i16_sp`] but with i32 audio resolution.
pub fn apply_lpf_i32_sp(data: &mut [i32], sample_rate_hz: u16, cutoff_frequency_hz: u16) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1.0 / (cutoff_frequency_hz as f32 * 2.0 * core::f32::consts::PI);
    let dt = 1.0 / sample_rate_hz as f32;
    let alpha = dt / (rc + dt);

    let mut cpy_orig_data = Vec::with_capacity(data.len());
    cpy_orig_data.extend_from_slice(data);

    data[0] = (alpha * cpy_orig_data[0] as f32) as i32;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        data[i] = (data[i - 1] as f32
            + alpha *
            (cpy_orig_data[i] as f32 - data[i-1] as f32)) as i32;
    }
}
