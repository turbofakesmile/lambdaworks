use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use lambdaworks_math::{
    field::fields::{
        fft_friendly::{
            stark_252_prime_field::{MontgomeryConfigStark252PrimeField, Stark252PrimeField},
            u64_mersenne_montgomery_field::MontgomeryConfigMersenne31PrimeField,
        },
        montgomery_backed_prime_fields::IsModulus,
    },
    unsigned_integer::{
        element::{U256, U64},
        montgomery::MontgomeryAlgorithms,
    },
};

mod utils;
use utils::{u32_mont_utils, u32_utils, u64_utils};

//TODO: split into files and subdirectory
pub fn u64_mersenne_montgomery_field_ops_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("U64 Mersenne Montgomery operations");
    let (x, y) = u32_mont_utils::get_field_elements();


    
    group.bench_with_input("pow", &(x, 5u64), |bench, (x, y)| {
        bench.iter(|| x.pow(black_box(*y)));
    });

}

pub fn mersenne31_field_ops_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mersenne31 operations");
    let (x, y) = u32_utils::get_field_elements();

    group.bench_with_input("pow", &(x, 5u64), |bench, (x, y)| {
        bench.iter(|| x.pow(black_box(*y)));
    });

}

criterion_group!(
    starkfp,
    u64_mersenne_montgomery_field_ops_benchmarks,
    mersenne31_field_ops_benchmark
);
criterion_main!(starkfp);
