use criterion::Criterion;
use raug::prelude::*;

const SAMPLE_RATE: f32 = 48_000.0;
const BLOCK_SIZES: &[usize] = &[128, 512, 2048];

fn name(name: &str) -> String {
    format!("{}_f32", name)
}

#[processor(derive(Default))]
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
    // create a new graph
    let mut graph = Graph::new();

    // add a sine oscillator
    let sine = graph.add_node(SineOscillator::default());
    let c440 = graph.constant(440.0);
    graph.connect(c440, 0, sine, 0);

    // add an output (mono)
    let out_l = graph.add_audio_output();
    graph.connect(sine, 0, out_l, 0);

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

pub fn bench_big_graph(c: &mut Criterion) {
    // create a new graph
    let mut graph = Graph::new();

    // add a sine oscillator
    let mut last_node = graph.add_node(SineOscillator::default());
    let c440 = graph.constant(440.0);
    graph.connect(c440, 0, last_node, 0);

    // add a lot of adders in series
    for _ in 0..1000 {
        let add = graph.add_node(Add::default());
        graph.connect(last_node, 0, add, 0);
        graph.connect(last_node, 0, add, 1);
        last_node = add;
    }

    // add some outputs (2 for stereo)
    let out_l = graph.add_audio_output();
    graph.connect(last_node, 0, out_l, 0);

    let mut group = c.benchmark_group(name("big_graph"));

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
