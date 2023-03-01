use criterion::{criterion_group, criterion_main, Criterion};

mod benchmarks;

fn run_all_benchmarks(c: &mut Criterion) {
    benchmarks::field::u64_benchmark(c);
    benchmarks::u384_field::u384_field_benchmark(c);
}

criterion_group!(benches, run_all_benchmarks);
criterion_main!(benches);
