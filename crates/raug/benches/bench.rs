use criterion::{criterion_group, criterion_main};

pub mod common;

criterion_group!(benches, common::bench_demo, common::bench_big_graph);
criterion_main!(benches);
