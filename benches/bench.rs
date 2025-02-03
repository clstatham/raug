use criterion::{criterion_group, criterion_main, Criterion};
use raug::prelude::*;

const SAMPLE_RATE: Float = 48_000.0;
const BLOCK_SIZES: &[usize] = &[128, 512, 2048];

fn name(name: &str) -> String {
    #[cfg(feature = "f32_samples")]
    {
        format!("{}_f32", name)
    }
    #[cfg(not(feature = "f32_samples"))]
    {
        format!("{}_f64", name)
    }
}

pub fn bench_demo(c: &mut Criterion) {
    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();

    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").connect(440.0);
    let sine = sine * 0.2;
    sine.output(0).connect(&out1.input(0));

    let mut runtime = graph.build_runtime();

    let mut group = c.benchmark_group(name("demo"));

    for &block_size in BLOCK_SIZES {
        runtime.allocate_for_block_size(SAMPLE_RATE, block_size);

        group.throughput(criterion::Throughput::Elements(block_size as u64));
        group.bench_function(format!("block_size_{}", block_size), |b| {
            b.iter(|| {
                runtime.process().unwrap();
            });
        });
    }

    group.finish();
}

// pub fn bench_generative1(c: &mut Criterion) {
//     let num_tones = 20;
//     let graph = generative1::generative1(num_tones);

//     let mut runtime = graph.build_runtime();

//     let mut group = c.benchmark_group(name(&format!("generative1_{}", num_tones)));

//     for &block_size in BLOCK_SIZES {
//         runtime.allocate_for_block_size(SAMPLE_RATE, block_size);

//         group.throughput(criterion::Throughput::Elements(block_size as u64));
//         group.bench_function(format!("block_size_{}", block_size), |b| {
//             b.iter(|| {
//                 runtime.process().unwrap();
//             });
//         });
//     }

//     group.finish();
// }

criterion_group!(
    benches, bench_demo,
    // bench_generative1
);
criterion_main!(benches);
