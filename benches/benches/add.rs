use std::{ops::Add, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use utils::generate_random_elements;

use crate::utils::to_lambdaworks_vec;

pub mod utils;

const BENCHMARK_NAME: &str = "add";

pub fn criterion_benchmark(c: &mut Criterion) {
    let arkworks_vec = generate_random_elements(10_000_000);

    // lambdaworks-math
    {
        let lambdaworks_vec = to_lambdaworks_vec(&arkworks_vec);

        c.bench_function(
            &format!("{} 10M elements cumulative | lambdaworks", BENCHMARK_NAME,),
            |b| {
                b.iter(|| {
                    let vals = &lambdaworks_vec[..];
                    let (mut i, end) = (1, vals.len() - 1);
                    let mut c = vals[0];
                    while i < end {
                        c = c.add(vals[i]);
                        i += 1;
                    }
                    black_box(c);
                });
            },
        );
    }
    // arkworks-ff
    {
        let arkworks_vec = arkworks_vec.clone();

        c.bench_function(
            &format!("{} 10M elements cumulative | ark-ff - ef8f758", BENCHMARK_NAME),
            |b| {
                b.iter(|| {
                    let vals = &arkworks_vec[..];
                    let (mut i, end) = (1, vals.len() - 1);
                    let mut c = vals[0];
                    while i < end {
                        c = c.add(vals[i]);
                        i += 1;
                    }
                    black_box(c);
                });
            },
        );
    }



    // lambdaworks-math
    {
        let lambdaworks_vec = to_lambdaworks_vec(&arkworks_vec);

        c.bench_function(
            &format!("{} 10M elements | lambdaworks", BENCHMARK_NAME,),
            |b| {
                b.iter(|| {
                    let vals = &lambdaworks_vec[..];
                    let (mut i, end) = (0, vals.len() - 1);
                    while i < end {
                        black_box(black_box(vals[i]).add(black_box(vals[i+1])));
                        i += 1;
                    }
                });
            },
        );
    }
    // arkworks-ff
    {
        let arkworks_vec = arkworks_vec.clone();

        c.bench_function(
            &format!("{} 10M elements | ark-ff - ef8f758", BENCHMARK_NAME),
            |b| {
                b.iter(|| {
                    let vals = &arkworks_vec[..];
                    let (mut i, end) = (0, vals.len() - 1);
                    while i < end {
                        black_box(black_box(vals[i]).add(black_box(vals[i+1])));
                        i += 1;
                    }
                });
            },
        );
    }
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default()
        .significance_level(0.01)
        .measurement_time(Duration::from_secs(15))
        .sample_size(300);
    targets = criterion_benchmark
}
criterion_main!(benches);
