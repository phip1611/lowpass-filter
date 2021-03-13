//! Double precision / f64 / double.

/// Applies a double precision (double, f64) low pass filter on a vector of **mono sample** in 16
/// bit resolution. If you have stereo data, call this function for each channel, convert it first
/// to mono or do whatever fits your use case.
///
/// ## Parameters
/// * `data` audio samples
/// * `sample_rate_hz` Sample Rate, e.g. 44100Hz
/// * `cutoff_frequency_hz` upper bound for frequencies to be cut, e.g. 150Hz
pub fn apply_lpf_i16_dp(data: &mut [i16], sample_rate_hz: u16, cutoff_frequency_hz: u16) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1.0 / (cutoff_frequency_hz as f64 * 2.0 * core::f64::consts::PI);
    let dt = 1.0 / sample_rate_hz as f64;
    let alpha = dt / (rc + dt);

    data[0] = (alpha * data[0] as f64) as i16;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter

        // we don't need a copy of the original data, because the original data is accessed
        // before it is overwritten: data[i] = ... data[i]

        data[i] =
            (data[i - 1] as f64 + alpha * (data[i] as f64 - data[i - 1] as f64)) as i16;
    }
}

/// Same as [`apply_lpf_i16_dp`] but with i32 audio resolution.
pub fn apply_lpf_i32_dp(data: &mut [i32], sample_rate_hz: u16, cutoff_frequency_hz: u16) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1.0 / (cutoff_frequency_hz as f64 * 2.0 * core::f64::consts::PI);
    let dt = 1.0 / sample_rate_hz as f64;
    let alpha = dt / (rc + dt);

    data[0] = (alpha * data[0] as f64) as i32;
    for i in 1..data.len() {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter

        // we don't need a copy of the original data, because the original data is accessed
        // before it is overwritten: data[i] = ... data[i]

        data[i] =
            (data[i - 1] as f64 + alpha * (data[i] as f64 - data[i - 1] as f64)) as i32;
    }
}
