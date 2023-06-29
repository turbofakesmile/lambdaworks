// TODO: generalize test functions for testing different fields (necesarry for MSM)
#[cfg(test)]
mod tests {
    use crate::{
        field::{
            element::FieldElement, fields::fft_friendly::stark_252_prime_field::Stark252PrimeField,
        },
        unsigned_integer::{element::U256, traits::U32Limbs},
    };
    use lambdaworks_gpu::metal::abstractions::state::MetalState;
    use metal::MTLSize;
    use proptest::prelude::*;

    pub type F = Stark252PrimeField;
    pub type FE = FieldElement<F>;
    pub type U = U256; // F::BaseType

    mod u256_tests {
        use super::*;

        enum BigOrSmallInt {
            Big(U),
            Small(usize),
        }

        fn execute_kernel(name: &str, params: (U, BigOrSmallInt)) -> U {
            let state = MetalState::new(None).unwrap();
            let pipeline = state.setup_pipeline(name).unwrap();

            let (a, b) = params;
            let a = a.to_u32_limbs();
            // conversion needed because of possible difference of endianess between host and
            // device (Metal's UnsignedInteger has 32bit limbs).

            let result_buffer = state.alloc_buffer::<U>(1);

            let (command_buffer, command_encoder) = match b {
                BigOrSmallInt::Big(b) => {
                    let b = b.to_u32_limbs();
                    let a_buffer = state.alloc_buffer_data(&a);
                    let b_buffer = state.alloc_buffer_data(&b);
                    state.setup_command(
                        &pipeline,
                        Some(&[(0, &a_buffer), (1, &b_buffer), (2, &result_buffer)]),
                    )
                }
                BigOrSmallInt::Small(b) => {
                    let a_buffer = state.alloc_buffer_data(&a);
                    let b_buffer = state.alloc_buffer_data(&[b]);
                    state.setup_command(
                        &pipeline,
                        Some(&[(0, &a_buffer), (1, &b_buffer), (2, &result_buffer)]),
                    )
                }
            };

            let threadgroup_size = MTLSize::new(1, 1, 1);
            let threadgroup_count = MTLSize::new(1, 1, 1);

            command_encoder.dispatch_thread_groups(threadgroup_count, threadgroup_size);
            command_encoder.end_encoding();

            command_buffer.commit();
            command_buffer.wait_until_completed();

            let limbs = MetalState::retrieve_contents::<u32>(&result_buffer);
            U::from_u32_limbs(&limbs)
        }

        prop_compose! {
            fn rand_u()(n in any::<u128>()) -> U { U::from_u128(n) } // doesn't populate all limbs
        }

        use BigOrSmallInt::{Big, Small};

        proptest! {
            #[test]
            fn add(a in rand_u(), b in rand_u()) {
                objc::rc::autoreleasepool(|| {
                    let result = execute_kernel("test_add_u256", (a, Big(b)));
                    prop_assert_eq!(result, a + b);
                    Ok(())
                }).unwrap();
            }

            #[test]
            fn sub(a in rand_u(), b in rand_u()) {
                objc::rc::autoreleasepool(|| {
                    let a = std::cmp::max(a, b);
                    let b = std::cmp::min(a, b);

                    let result = execute_kernel("test_sub_u256", (a, Big(b)));
                    prop_assert_eq!(result, a - b);
                    Ok(())
                }).unwrap();
            }

            #[test]
            fn prod(a in rand_u(), b in rand_u()) {
                objc::rc::autoreleasepool(|| {
                    let result = execute_kernel("test_mul_u256", (a, Big(b)));
                    prop_assert_eq!(result, a * b);
                    Ok(())
                }).unwrap();
            }

            #[test]
            fn shl(a in rand_u(), b in any::<usize>()) {
                objc::rc::autoreleasepool(|| {
                    let b = b % 256; // so it doesn't overflow
                    let result = execute_kernel("test_shl_u256", (a, Small(b)));
                    prop_assert_eq!(result, a << b);
                    Ok(())
                }).unwrap();
            }

            #[test]
            fn shr(a in rand_u(), b in any::<usize>()) {
                objc::rc::autoreleasepool(|| {
                    let b = b % 256; // so it doesn't overflow
                    let result = execute_kernel("test_shr_u256", (a, Small(b)));
                    prop_assert_eq!(result, a >> b);
                    Ok(())
                }).unwrap();
            }
        }
    }

    mod fp_tests {
        use proptest::collection;

        use super::*;

        enum FEOrInt {
            Elem(FE),
            Int(u32),
        }

        fn execute_kernel(name: &str, a: &FE, b: FEOrInt) -> FE {
            let state = MetalState::new(None).unwrap();
            let pipeline = state.setup_pipeline(name).unwrap();

            // conversion needed because of possible difference of endianess between host and
            // device (Metal's UnsignedInteger has 32bit limbs).
            let a = a.value().to_u32_limbs();
            let result_buffer = state.alloc_buffer::<u32>(12);

            let (command_buffer, command_encoder) = match b {
                FEOrInt::Elem(b) => {
                    let b = b.value().to_u32_limbs();
                    let a_buffer = state.alloc_buffer_data(&a);
                    let b_buffer = state.alloc_buffer_data(&b);

                    state.setup_command(
                        &pipeline,
                        Some(&[(0, &a_buffer), (1, &b_buffer), (2, &result_buffer)]),
                    )
                }
                FEOrInt::Int(b) => {
                    let a_buffer = state.alloc_buffer_data(&a);
                    let b_buffer = state.alloc_buffer_data(&[b]);

                    state.setup_command(
                        &pipeline,
                        Some(&[(0, &a_buffer), (1, &b_buffer), (2, &result_buffer)]),
                    )
                }
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

        prop_compose! {
            fn rand_u32()(n in any::<u32>()) -> u32 { n }
        }

        prop_compose! {
            fn rand_limbs()(vec in collection::vec(rand_u32(), 12)) -> Vec<u32> {
                vec
            }
        }

        prop_compose! {
            fn rand_felt()(limbs in rand_limbs()) -> FE {
                FE::from(&U::from_u32_limbs(&limbs))
            }
        }

        use FEOrInt::{Elem, Int};

        proptest! {
            #[test]
            fn add(a in rand_felt(), b in rand_felt()) {
                objc::rc::autoreleasepool(|| {
                    let result = execute_kernel("test_add_stark256", &a, Elem(b.clone()));
                    prop_assert_eq!(result, a + b);
                    Ok(())
                }).unwrap();
            }

            #[test]
            fn sub(a in rand_felt(), b in rand_felt()) {
                objc::rc::autoreleasepool(|| {
                    let result = execute_kernel("test_sub_stark256", &a, Elem(b.clone()));
                    prop_assert_eq!(result, a - b);
                    Ok(())
                }).unwrap();
            }

            #[test]
            fn mul(a in rand_felt(), b in rand_felt()) {
                objc::rc::autoreleasepool(|| {
                    let result = execute_kernel("test_mul_stark256", &a, Elem(b.clone()));
                    prop_assert_eq!(result, a * b);
                    Ok(())
                }).unwrap();
            }

            #[test]
            fn pow(a in rand_felt(), b in rand_u32()) {
                objc::rc::autoreleasepool(|| {
                    let result = execute_kernel("test_pow_stark256", &a, Int(b));
                    prop_assert_eq!(result, a.pow(b));
                    Ok(())
                }).unwrap();
            }
        }
    }
}
