use criterion::{black_box, Criterion};
use lambdaworks_math::{
    elliptic_curve::short_weierstrass::curves::bls12_381::field_extension::BLS12381PrimeField,
    field::element::FieldElement, unsigned_integer::element::U384,
};
use rand::Rng;

type E = FieldElement<BLS12381PrimeField>;

fn add(x: E, y: E) -> E {
    x + y
}

fn mul(x: E, y: E) -> E {
    x * y
}

pub fn u384_field_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let mut group = c.benchmark_group("u384_primefield");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    group.significance_level(0.1).sample_size(1000);

    group.bench_function("add", |b| {
        let x = E::new(U384 {
            limbs: [
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
            ],
        });
        let y = E::new(U384 {
            limbs: [
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
            ],
        });
        b.iter(|| add(black_box(x.clone()), black_box(y.clone())))
    });

    group.bench_function("mul", |b| {
        let x = E::new(U384 {
            limbs: [
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
            ],
        });
        let y = E::new(U384 {
            limbs: [
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
            ],
        });
        b.iter(|| mul(black_box(x.clone()), black_box(y.clone())))
    });
}
