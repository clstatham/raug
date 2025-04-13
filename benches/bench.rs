use criterion::{Criterion, criterion_group, criterion_main};
use raug::prelude::*;

mod generative1;

const SAMPLE_RATE: f32 = 48_000.0;
const BLOCK_SIZES: &[usize] = &[128, 512, 2048];

fn name(name: &str) -> String {
    format!("{}_f32", name)
}

#[processor]
pub fn sine_oscillator(
    env: ProcEnv,
    #[state] phase: &mut f32,
    #[input] freq: &f32,
    #[output] out: &mut f32,
) -> ProcResult<()> {
    *phase += 2.0 * std::f32::consts::PI * freq / env.sample_rate;
    *out = phase.sin() * 0.2;
    Ok(())
}

pub fn bench_demo(c: &mut Criterion) {
    let graph = Graph::new();

    let out1 = graph.add_audio_output();

    let sine = graph.add(SineOscillator {
        phase: 0.0,
        freq: 440.0,
        out: 0.0,
    });
    sine.output(0).connect(&out1.input(0));

    let mut group = c.benchmark_group(name("demo"));

    for &block_size in BLOCK_SIZES {
        graph.allocate(SAMPLE_RATE, block_size);

        group.throughput(criterion::Throughput::Elements(block_size as u64));
        group.bench_function(format!("block_size_{}", block_size), |b| {
            b.iter(|| {
                graph.process().unwrap();
            });
        });
    }

    group.finish();
}

fn bench_generative1(c: &mut Criterion) {
    let num_tones = 20;
    let graph = generative1::generative1(num_tones);
    let mut group = c.benchmark_group(name(&format!("generative1_{}", num_tones)));

    for &block_size in BLOCK_SIZES {
        graph.allocate(SAMPLE_RATE, block_size);

        group.throughput(criterion::Throughput::Elements(block_size as u64));
        group.bench_function(format!("block_size_{}", block_size), |b| {
            b.iter(|| {
                graph.process().unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_generative1);
criterion_main!(benches);
