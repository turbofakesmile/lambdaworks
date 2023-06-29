#![no_main]
use lambdaworks_gpu::metal::abstractions::state::MetalState;
use lambdaworks_math::unsigned_integer::element::UnsignedInteger;
use lambdaworks_math::{
    fft::{
        cpu::{ops::fft as fft_cpu, roots_of_unity::get_twiddles},
        gpu::metal::ops::fft as fft_metal,
    },
    field::{
        element::FieldElement, fields::fft_friendly::stark_252_prime_field::Stark252PrimeField,
        traits::RootsConfig,
    },
    unsigned_integer::{element::U256, traits::U32Limbs},
};
use libfuzzer_sys::fuzz_target;
use metal::MTLSize;

type F = Stark252PrimeField;
type FE = FieldElement<F>;
type U = U256;
const NUM_LIMBS: usize = 8;

fuzz_target!(|data: ([u32; 8], [u32; 8])| {
    let (a_raw, b_raw) = data;
    let (a, b) = {
        let a_u = U::from_u32_limbs(&a_raw);
        let b_u = U::from_u32_limbs(&b_raw);

        (FE::from(&a_u), FE::from(&b_u))
    };

    let res = execute_kernel("test_mul_stark256", &a, &b);
    assert_eq!(res, a * b);
});

fn execute_kernel(name: &str, a: &FE, b: &FE) -> FE {
    let state = MetalState::new(None).unwrap();
    let pipeline = state.setup_pipeline(name).unwrap();

    // conversion needed because of possible difference of endianess between host and
    // device (Metal's UnsignedInteger has 32bit limbs).
    let a = a.value().to_u32_limbs();
    let result_buffer = state.alloc_buffer::<u32>(NUM_LIMBS);

    let (command_buffer, command_encoder) = {
        let b = b.value().to_u32_limbs();
        let a_buffer = state.alloc_buffer_data(&a);
        let b_buffer = state.alloc_buffer_data(&b);

        state.setup_command(
            &pipeline,
            Some(&[(0, &a_buffer), (1, &b_buffer), (2, &result_buffer)]),
        )
    };

    let threadgroup_size = MTLSize::new(1, 1, 1);
    let threadgroup_count = MTLSize::new(1, 1, 1);

    command_encoder.dispatch_thread_groups(threadgroup_count, threadgroup_size);
    command_encoder.end_encoding();

    command_buffer.commit();
    command_buffer.wait_until_completed();

    let limbs = MetalState::retrieve_contents::<u32>(&result_buffer);
    FE::from_raw(&U::from_u32_limbs(&limbs))
}
