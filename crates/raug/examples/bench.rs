include!("../benches/common.rs");

fn main() {
    let mut criterion = Criterion::default();
    bench_demo(&mut criterion);
    bench_big_graph(&mut criterion);
}
