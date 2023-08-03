use std::{ops::Add, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use utils::generate_random_elements;

use crate::utils::to_lambdaworks_vec;

pub mod utils;

const BENCHMARK_NAME: &str = "add";

pub fn criterion_benchmark(c: &mut Criterion) {
    let arkworks_vec = generate_random_elements(10_000_000);

    // arkworks-ff
    {
        c.bench_function(
            &format!("{} 10M elements | ark-ff - ef8f758", BENCHMARK_NAME),
            |b| {
                b.iter(|| {
                    let mut iter = arkworks_vec.iter();
                    let a = iter.next().unwrap();
                    let b = iter.next().unwrap();
                    let mut c = a.add(b);

                    for _i in 2..10_000_000 {
                        let a = iter.next().unwrap();
                        c = c.add(a);
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
                    let mut iter = lambdaworks_vec.iter();
                    let a = iter.next().unwrap();
                    let b = iter.next().unwrap();
                    let mut c = a.add(b);

                    for _i in 2..10_000_000 {
                        let a = iter.next().unwrap();
                        c =c.add(a);
                    }
                    black_box(c);
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
