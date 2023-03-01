use criterion::{criterion_group, criterion_main, Criterion};

mod benchmarks;

fn run_field_benchmarks(c: &mut Criterion) {
    benchmarks::field::u64_benchmark(c);
    benchmarks::u384_field::u384_field_benchmark(c);
}

criterion_group!(benches, run_field_benchmarks);
criterion_main!(benches);
