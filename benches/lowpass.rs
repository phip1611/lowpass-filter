use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use lowpass_filter::LowpassFilter;
use std::hint::black_box;

const SAMPLE_RATE_HZ: f64 = 44100.0;
/// One second of audio at [`SAMPLE_RATE_HZ`].
const SAMPLE_COUNT: usize = SAMPLE_RATE_HZ as usize;
/// Frequency of the generated sine wave.
const FREQUENCY_HZ: f64 = 70.0;
/// Amplitude of the generated sine wave, keeping samples in `-1.0..=1.0`.
const AMPLITUDE: f64 = 0.8;
/// Typical buffer size of a real-time audio callback.
const STREAM_BUFFER_SIZE: usize = 64;

fn benchmark(c: &mut Criterion) {
    // Generate a sine wave
    let samples_f64 = (0..SAMPLE_COUNT)
        .map(|i| i as f64 / SAMPLE_RATE_HZ)
        .map(|t| (t * FREQUENCY_HZ * 2.0 * core::f64::consts::PI).sin() * AMPLITUDE)
        .collect::<Vec<_>>();
    let samples_f32 = samples_f64.iter().map(|&x| x as f32).collect::<Vec<_>>();

    let mut group = c.benchmark_group("lowpass_filter");
    group.throughput(Throughput::Elements(SAMPLE_COUNT as u64));
    group.bench_function("f32", |b| {
        b.iter_batched_ref(
            || samples_f32.clone(),
            |samples| {
                let mut filter = LowpassFilter::<f32>::new(44100.0, 120.0);
                for sample in black_box(samples.as_mut_slice()) {
                    *sample = filter.run(*sample);
                }
            },
            BatchSize::LargeInput,
        )
    });
    group.bench_function("f64", |b| {
        b.iter_batched_ref(
            || samples_f64.clone(),
            |samples| {
                let mut filter = LowpassFilter::<f64>::new(44100.0, 120.0);
                for sample in black_box(samples.as_mut_slice()) {
                    *sample = filter.run(*sample);
                }
            },
            BatchSize::LargeInput,
        )
    });
    group.bench_function("f32 slice", |b| {
        b.iter_batched_ref(
            || samples_f32.clone(),
            |samples| {
                LowpassFilter::<f32>::new(44100.0, 120.0)
                    .run_slice(black_box(samples.as_mut_slice()));
            },
            BatchSize::LargeInput,
        )
    });
    // common case, filter is reused across many small audio buffers
    group.bench_function("f32 slice (streaming)", |b| {
        let mut filter = LowpassFilter::<f32>::new(44100.0, 120.0);
        b.iter_batched_ref(
            || samples_f32.clone(),
            |samples| {
                for buffer in black_box(samples.as_mut_slice()).chunks_mut(STREAM_BUFFER_SIZE) {
                    filter.run_slice(buffer);
                }
            },
            BatchSize::LargeInput,
        )
    });
    group.bench_function("f64 slice", |b| {
        b.iter_batched_ref(
            || samples_f64.clone(),
            |samples| {
                LowpassFilter::<f64>::new(44100.0, 120.0)
                    .run_slice(black_box(samples.as_mut_slice()));
            },
            BatchSize::LargeInput,
        )
    });
    // common case, filter is reused across many small audio buffers
    group.bench_function("f64 slice (streaming)", |b| {
        let mut filter = LowpassFilter::<f64>::new(44100.0, 120.0);
        b.iter_batched_ref(
            || samples_f64.clone(),
            |samples| {
                for buffer in black_box(samples.as_mut_slice()).chunks_mut(STREAM_BUFFER_SIZE) {
                    filter.run_slice(buffer);
                }
            },
            BatchSize::LargeInput,
        )
    });
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
