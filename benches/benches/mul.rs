use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::ops::Mul;
use utils::generate_random_elements;

use crate::utils::to_lambdaworks_vec;

pub mod utils;

const BENCHMARK_NAME: &str = "mul";

pub fn criterion_benchmark(c: &mut Criterion) {
    let arkworks_vec = generate_random_elements(2_000_000);

    // arkworks-ff
    {
        c.bench_function(
            &format!(
                "{} 2M elements | ark-ff - commit: ef8f758 ",
                BENCHMARK_NAME
            ),
            |b| {
                b.iter(|| {
                    let mut iter = arkworks_vec.iter();
                    let a = iter.next().unwrap();
                    let b = iter.next().unwrap();
                    let mut c = a.mul(b);

                    for _i in 2..2_000_000 {
                        let a = iter.next().unwrap();
                        c = c.mul(a);
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
            &format!("{} 2M elements | lambdaworks", BENCHMARK_NAME,),
            |b| {
                b.iter(|| {
                    let mut iter = lambdaworks_vec.iter();
                    let a = iter.next().unwrap();
                    let b = iter.next().unwrap();
                    let mut c = a.mul(b);

                    for _i in 2..2_000_000 {
                        let a = iter.next().unwrap();
                        c =c.mul(a);
                    }
                    black_box(c);
                });
            },
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
