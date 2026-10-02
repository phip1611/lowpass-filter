use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type};
use criterion::measurement::WallTime;
use criterion::{
    AxisScale, BatchSize, BenchmarkGroup, BenchmarkId, Criterion, PlotConfiguration, Throughput,
    criterion_group, criterion_main,
};
use lowpass_filter::{LowpassFilter, Sample};
use std::hint::black_box;
use std::time::Duration;

const SAMPLE_RATE_HZ: f64 = 44100.0;
const CUTOFF_HZ: f64 = 120.0;
/// Frequency of the generated sine wave.
const FREQUENCY_HZ: f64 = 70.0;
/// Amplitude of the generated sine wave, keeping samples in `-1.0..=1.0`.
const AMPLITUDE: f64 = 0.8;
/// Input lengths for oneshot filtering: a small buffer, where the constructor
/// cost shows, and one second of audio for the throughput on long inputs.
const ONESHOT_LENGTHS: [usize; 2] = [64, 44100];
/// Typical buffer size of a real-time audio callback.
const STREAM_BUFFER_SIZES: [usize; 1] = [64];
/// Samples processed per streaming iteration: one second of audio.
const STREAM_SAMPLE_COUNT: usize = SAMPLE_RATE_HZ as usize;

/// Glue to write the benchmarks once for `f32` and `f64`.
trait BenchSample: Sample {
    const NAME: &'static str;
    fn from_f64(x: f64) -> Self;
    fn biquad() -> impl Biquad<Self>;
}

macro_rules! impl_bench_sample {
    ($t:ty) => {
        impl BenchSample for $t {
            const NAME: &'static str = stringify!($t);

            fn from_f64(x: f64) -> Self {
                x as $t
            }

            fn biquad() -> impl Biquad<Self> {
                let coeffs = Coefficients::<$t>::from_params(
                    Type::SinglePoleLowPass,
                    (SAMPLE_RATE_HZ as $t).hz(),
                    (CUTOFF_HZ as $t).hz(),
                    // Ignored by single-pole filters, but must be positive.
                    1.0,
                )
                .expect("should accept parameters below Nyquist");
                DirectForm2Transposed::<$t>::new(coeffs)
            }
        }
    };
}

impl_bench_sample!(f32);
impl_bench_sample!(f64);

fn new_filter<T: BenchSample>() -> LowpassFilter<T> {
    LowpassFilter::new(T::from_f64(SAMPLE_RATE_HZ), T::from_f64(CUTOFF_HZ))
}

/// Per-sample API.
fn run<T: BenchSample>(filter: &mut LowpassFilter<T>, samples: &mut [T]) {
    for sample in samples {
        *sample = filter.run(*sample);
    }
}

fn run_biquad<T: Copy>(biquad: &mut impl Biquad<T>, samples: &mut [T]) {
    for sample in samples {
        *sample = biquad.run(*sample);
    }
}

fn sine<T: BenchSample>(len: usize) -> Vec<T> {
    (0..len)
        .map(|i| i as f64 / SAMPLE_RATE_HZ)
        .map(|t| (t * FREQUENCY_HZ * 2.0 * core::f64::consts::PI).sin() * AMPLITUDE)
        .map(T::from_f64)
        .collect()
}

#[derive(Clone, Copy)]
enum Mode {
    /// Fresh filter per input.
    Oneshot,
    /// One filter reused across many small buffers, the common case in
    /// real-time audio.
    Streaming { buffer_size: usize },
}

/// Registers all filter variants. New variants only need to be added here.
fn bench_variants<T: BenchSample>(group: &mut BenchmarkGroup<WallTime>, mode: Mode, input: &[T]) {
    bench_variant(group, mode, input, "run", new_filter, run);
    bench_variant(
        group,
        mode,
        input,
        "run_slice",
        new_filter,
        LowpassFilter::run_slice,
    );
    bench_variant(group, mode, input, "biquad", T::biquad, run_biquad);
}

/// Times `process` on a fresh copy of `input` per iteration.
///
/// Filtering the same buffer over and over would attenuate the signal into
/// subnormal floats, which are much slower on most CPUs.
fn bench_variant<T: Clone, F>(
    group: &mut BenchmarkGroup<WallTime>,
    mode: Mode,
    input: &[T],
    name: &str,
    new: impl Fn() -> F,
    process: impl Fn(&mut F, &mut [T]),
) {
    let param = match mode {
        Mode::Oneshot => input.len(),
        Mode::Streaming { buffer_size } => buffer_size,
    };
    // Streaming state carries over between iterations, like in an audio
    // callback.
    let mut filter = new();
    group.bench_with_input(BenchmarkId::new(name, param), input, |b, input| {
        b.iter_batched_ref(
            || input.to_vec(),
            |samples| {
                let samples = black_box(samples.as_mut_slice());
                match mode {
                    Mode::Oneshot => process(&mut new(), samples),
                    Mode::Streaming { buffer_size } => {
                        for buffer in samples.chunks_mut(buffer_size) {
                            process(&mut filter, buffer);
                        }
                    }
                }
            },
            BatchSize::LargeInput,
        )
    });
}

fn bench_oneshot<T: BenchSample>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("oneshot/{}", T::NAME));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    for len in ONESHOT_LENGTHS {
        group.throughput(Throughput::Elements(len as u64));
        bench_variants(&mut group, Mode::Oneshot, &sine::<T>(len));
    }
    group.finish();
}

fn bench_streaming<T: BenchSample>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("streaming/{}", T::NAME));
    group.throughput(Throughput::Elements(STREAM_SAMPLE_COUNT as u64));
    let input = sine::<T>(STREAM_SAMPLE_COUNT);
    for buffer_size in STREAM_BUFFER_SIZES {
        bench_variants(&mut group, Mode::Streaming { buffer_size }, &input);
    }
    group.finish();
}

fn benchmark(c: &mut Criterion) {
    bench_oneshot::<f32>(c);
    bench_oneshot::<f64>(c);
    bench_streaming::<f32>(c);
    bench_streaming::<f64>(c);
}

criterion_group! {
    name = benches;
    // Shorter than the defaults to keep a full run at a few minutes.
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = benchmark
}
criterion_main!(benches);
