use super::traits::IsCryptoHash;
use lambdaworks_math::{
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{curve::BLS12381Curve, field_extension::BLS12381PrimeField},
            element::ProjectivePoint,
        },
        traits::IsEllipticCurve,
    },
    field::{self, element::FieldElement, traits::IsField},
    traits::ByteConversion,
    unsigned_integer::element::U384,
};

type FE = field::element::FieldElement<BLS12381PrimeField>;
type Point = ProjectivePoint<BLS12381Curve>;

const WINDOW_SIZE: usize = 4;
const NUM_WINDOWS: usize = 96;
const INPUT_SIZE_IN_BITS: usize = WINDOW_SIZE * NUM_WINDOWS;
const HALF_INPUT_SIZE_BITS: usize = INPUT_SIZE_IN_BITS / 2;

// TODO: this function should be replaced with the trait method once it is implemented.
pub fn random_field_element<F>(rng: &mut rand::rngs::ThreadRng) -> FieldElement<F>
where
    F: field::traits::IsField,
{
    FieldElement::<F>::from(rand::Rng::gen::<u64>(rng))
}

pub struct Pedersen<E>
where
    E: IsEllipticCurve,
{
    parameters: Vec<Vec<FieldElement<E::BaseField>>>,
}

impl<E> Pedersen<E>
where
    E: IsEllipticCurve,
{
    pub fn from_parameters(parameters: Vec<Vec<FieldElement<E::BaseField>>>) -> Self {
        Self { parameters }
    }

    fn create_generators(rng: &mut rand::rngs::ThreadRng) -> Vec<Vec<FieldElement<E::BaseField>>> {
        (0..NUM_WINDOWS)
            .into_iter()
            .map(|_| Self::generator_powers(WINDOW_SIZE, rng))
            .collect()
    }

    fn generator_powers(
        num_powers: usize,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Vec<FieldElement<E::BaseField>> {
        let base = random_field_element::<E::BaseField>(rng);
        (0..num_powers)
            .into_iter()
            .map(|exponent| base.pow(exponent))
            .collect()
    }
}

impl IsCryptoHash<BLS12381PrimeField> for Pedersen<BLS12381Curve> {
    fn new() -> Self {
        Self {
            parameters: Self::create_generators(&mut rand::thread_rng()),
        }
    }

    fn hash_one(&self, input: FE) -> FE {
        // Compute sum of h_i^{m_i} for all i.
        let bits = to_bits(input);
        bits.chunks(WINDOW_SIZE)
            .zip(&self.parameters)
            .map(|(bits, generator_powers)| {
                let mut encoded = FE::zero();
                for (bit, base) in bits.iter().zip(generator_powers.iter()) {
                    if *bit {
                        encoded += base.clone();
                    }
                }
                encoded
            })
            // This last step is the same as doing .sum() but std::iter::Sum is
            // not implemented for FieldElement yet.
            .fold(FE::zero(), |acc, x| acc + x)
    }

    fn hash_two(&self, left: FE, right: FE) -> FE {
        let left_input_bytes = left.value().to_bytes_be();
        let right_input_bytes = right.value().to_bytes_be();
        let mut buffer = vec![0u8; (HALF_INPUT_SIZE_BITS + HALF_INPUT_SIZE_BITS) / 8];

        buffer
            .iter_mut()
            .zip(left_input_bytes.iter().chain(right_input_bytes.iter()))
            .for_each(|(b, l_b)| *b = *l_b);

        let base_type_value = U384::from_bytes_be(&buffer).unwrap();
        let new_input_value = BLS12381PrimeField::from_base_type(base_type_value);
        let new_input = FE::from(&new_input_value);

        self.hash_one(new_input)
    }
}

fn to_bits(felt: FE) -> Vec<bool> {
    let felt_bytes = felt.value().to_bytes_be();
    let mut bits = Vec::with_capacity(felt_bytes.len() * 8);
    for byte in felt_bytes {
        for i in 0..8 {
            bits.push(byte & (1 << i) != 0);
        }
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{pedersen::Pedersen, traits::IsCryptoHash};
    use lambdaworks_math::{
        elliptic_curve::short_weierstrass::traits::IsShortWeierstrass,
        field::{
            element::FieldElement,
            fields::u384_prime_field::{IsMontgomeryConfiguration, MontgomeryBackendPrimeField},
        },
        unsigned_integer::element::{U256, U384},
    };

    // Define starkware-rs field and field element.
    #[derive(Clone, Debug)]
    pub struct TestFieldConfig;
    impl IsMontgomeryConfiguration for TestFieldConfig {
        const MODULUS: U384 =
            U384::from("800000000000011000000000000000000000000000000000000000000000001");
        const MP: u64 = 18446744073709551615u64;
        const R2: U384 =
            U384::from("38E5F79873C0A6DF47D84F8363000187545706677FFCC06CC7177D1406DF18E");
    }

    type PedersenTestField = MontgomeryBackendPrimeField<TestFieldConfig>;
    type TestFieldElement = FieldElement<PedersenTestField>;

    // Define starkware-rs elliptic curve.
    #[derive(Clone, Debug)]
    pub struct TestCurve;

    impl IsShortWeierstrass for TestCurve {
        type UIntOrders = U384;

        fn a() -> FieldElement<Self::BaseField> {
            let alpha = Self::UIntOrders {
                limbs: [
                    0,
                    0,
                    18446744073709551585,
                    18446744073709551615,
                    18446744073709551615,
                    576460752303422960,
                ],
            };
            FieldElement::<Self::BaseField>::from(&alpha)
        }

        fn b() -> FieldElement<Self::BaseField> {
            let beta = Self::UIntOrders {
                limbs: [
                    0,
                    0,
                    3863487492851900874,
                    7432612994240712710,
                    12360725113329547591,
                    88155977965380735,
                ],
            };
            FieldElement::<Self::BaseField>::from(&beta)
        }

        fn order_r() -> Self::UIntOrders {
            todo!()
        }

        fn order_p() -> Self::UIntOrders {
            todo!()
        }

        fn target_normalization_power() -> Vec<u64> {
            todo!()
        }
    }

    impl IsEllipticCurve for TestCurve {
        type BaseField = PedersenTestField;
        type PointRepresentation = TestCurveProjectivePoint;

        fn generator() -> Self::PointRepresentation {
            let x = FieldElement::<Self::BaseField>::from(&U384 {
                limbs: [
                    0,
                    0,
                    14484022957141291997,
                    5884444832209845738,
                    299981207024966779,
                    232005955912912577,
                ],
            });
            let y = FieldElement::<Self::BaseField>::from(&U384 {
                limbs: [
                    0,
                    0,
                    6241159653446987914,
                    664812301889158119,
                    18147424675297964973,
                    405578048423154473,
                ],
            });
            Self::create_point_from_affine(x, y)
        }

        fn create_point_from_affine(
            x: FieldElement<Self::BaseField>,
            y: FieldElement<Self::BaseField>,
        ) -> Self::PointRepresentation {
            Self::PointRepresentation::new([x, y, FieldElement::one()])
        }
    }

    fn to_bits(felt: TestFieldElement) -> Vec<bool> {
        let felt_bytes = felt.value().to_bytes_be();
        let mut bits = Vec::with_capacity(felt_bytes.len() * 8);
        for byte in felt_bytes {
            for i in 0..8 {
                bits.push(byte & (1 << i) != 0);
            }
        }
        bits
    }

    impl IsCryptoHash<PedersenTestField> for Pedersen<TestCurve> {
        fn new() -> Self {
            Self {
                parameters: Self::create_generators(&mut rand::thread_rng()),
            }
        }

        fn hash_one(&self, input: TestFieldElement) -> TestFieldElement {
            // Compute sum of h_i^{m_i} for all i.
            let bits = to_bits(input);
            bits.chunks(WINDOW_SIZE)
                .zip(&self.parameters)
                .map(|(bits, generator_powers)| {
                    let mut encoded = TestFieldElement::zero();
                    for (bit, base) in bits.iter().zip(generator_powers.iter()) {
                        if *bit {
                            encoded += base.clone();
                        }
                    }
                    encoded
                })
                // This last step is the same as doing .sum() but std::iter::Sum is
                // not implemented for FieldElement yet.
                .fold(TestFieldElement::zero(), |acc, x| acc + x)
        }

        fn hash_two(&self, left: TestFieldElement, right: TestFieldElement) -> TestFieldElement {
            let left_input_bytes = left.value().to_bytes_be()[48 - 32..].to_vec();
            let right_input_bytes = right.value().to_bytes_be()[48 - 32..].to_vec();
            let mut buffer = vec![0u8; (256) / 8];

            buffer
                .iter_mut()
                .zip(left_input_bytes.iter().chain(right_input_bytes.iter()))
                .for_each(|(b, l_b)| *b = *l_b);
            for _ in 0..16 {
                buffer.insert(0, 0);
            }
            let base_type_value = U384::from_bytes_be(&buffer).unwrap();
            let new_input_value = PedersenTestField::from_base_type(base_type_value);
            let new_input = TestFieldElement::from(&new_input_value);

            self.hash_one(new_input)
        }
    }

    type TestCurveProjectivePoint = ProjectivePoint<TestCurve>;

    #[test]
    fn test_pedersen_hash() {
        let in1 = TestFieldElement::new(U384::from(
            "03d937c035c878245caf64531a5756109c53068da139362728feb561405371cb",
        ));
        let in2 = TestFieldElement::new(U384::from(
            "0208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a",
        ));
        let expected_hash = TestFieldElement::new(U384::from(
            "030e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662",
        ));

        let pederse_p0 = TestCurve::create_point_from_affine(
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    3602345268353203007,
                    13758484295849329960,
                    518715844721862878,
                    241691544791834578,
                ]
            }),
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    13441546676070136227,
                    13001553326386915570,
                    433857700841878496,
                    368891789801938570,
                ]
            }),
        );

        let pederse_p1 = TestCurve::create_point_from_affine(
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    16491878934996302286,
                    12382025591154462459,
                    10043949394709899044,
                    253000153565733272,
                ]
            }),
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    13950428914333633429,
                    2545498000137298346,
                    5191292837124484988,
                    285630633187035523,
                ]
            }),
        );

        let pederse_p2 = TestCurve::create_point_from_affine(
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    1203723169299412240,
                    18195981508842736832,
                    12916675983929588442,
                    338510149841406402,
                ]
            }),
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    12352616181161700245,
                    11743524503750604092,
                    11088962269971685343,
                    161068411212710156,
                ]
            }),
        );

        let pederse_p3 = TestCurve::create_point_from_affine(
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    1145636535101238356,
                    10664803185694787051,
                    299781701614706065,
                    425493972656615276,
                ]
            }),
            TestFieldElement::from(&U384 {
                limbs: [
                    0,
                    0,
                    8187986478389849302,
                    4428713245976508844,
                    6033691581221864148,
                    345457391846365716,
                ]
            }),
        );

        let parameters = vec![pederse_p0, pederse_p1, pederse_p2, pederse_p3];

        // let hasher: Pedersen<TestCurve> = Pedersen::from_parameters(parameters);

        // assert_eq!(hasher.hash_two(in1, in2), expected_hash);
    }
}
