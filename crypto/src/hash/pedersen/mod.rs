use super::traits::IsCryptoHash;
use lambdaworks_math::{
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{curve::BLS12381Curve, field_extension::BLS12381PrimeField},
        },
        traits::IsEllipticCurve//, point::ProjectivePoint,
    },
    field::{self, element::FieldElement, traits::IsField},
    traits::ByteConversion,
    unsigned_integer::element::U384,
};

type FE = field::element::FieldElement<BLS12381PrimeField>;
//type Point = ProjectivePoint<BLS12381Curve>;

const WINDOW_SIZE: usize = 4;
const NUM_WINDOWS: usize = 128;
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
        elliptic_curve::{
            traits::IsEllipticCurve, short_weierstrass::{traits::IsShortWeierstrass, point::ShortWeierstrassProjectivePoint, curves::bls12_377::curve::BLS12377Curve},
        },
        field::{
            element::FieldElement,
            fields::u384_prime_field::{IsMontgomeryConfiguration, MontgomeryBackendPrimeField},
        },
        unsigned_integer::element::{U384},
    };

    #[derive(Clone, Debug)]
    pub struct TestFieldConfig;
    impl IsMontgomeryConfiguration for TestFieldConfig {
        const MODULUS: U384 =
            U384::from("800000000000011000000000000000000000000000000000000000000000001");
        const MP: u64 = 18446744073709551615;
        const R2: U384 = U384::from("38e5f79873c0a6df47d84f8363000187545706677ffcc06cc7177d1406df18e");
    }
    
    type PedersenTestField = MontgomeryBackendPrimeField<TestFieldConfig>;
    type TestFieldElement = FieldElement<PedersenTestField>;
    
    // Define starkware-rs elliptic curve.
    #[derive(Clone, Debug)]
    pub struct TestCurve;
    
    impl IsShortWeierstrass for TestCurve {
        fn a() -> FieldElement<Self::BaseField> {
            let alpha = U384::from_u64(1);
            FieldElement::<Self::BaseField>::new(alpha)
        }
    
        fn b() -> FieldElement<Self::BaseField> {
            let beta = U384::from("6f21413efbe40de150e596d72f7a8c5609ad26c15c915c1f4cdfcb99cee9e89");
            FieldElement::<Self::BaseField>::new(beta)
        }
    }
    
    impl IsEllipticCurve for TestCurve {
        type BaseField = PedersenTestField;
        type PointRepresentation = ShortWeierstrassProjectivePoint<TestCurve>;
    
        fn generator() -> ShortWeierstrassProjectivePoint<TestCurve> {
            let x = FieldElement::<Self::BaseField>::new(U384::from("1ef15c18599971b7beced415a40f0c7deacfd9b0d1819e03d723d8bc943cfca"));
            let y = FieldElement::<Self::BaseField>::new(U384::from("5668060aa49730b7be4801df46ec62de53ecd11abe43a32873000c36e8dc1f"));
            Self::create_point_from_affine(x, y).unwrap()
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
            let left_input_bytes = left.value().to_bytes_be();
            let right_input_bytes = right.value().to_bytes_be();
            let mut buffer = vec![0u8; (HALF_INPUT_SIZE_BITS + HALF_INPUT_SIZE_BITS) / 8];
    
            buffer
                .iter_mut()
                .zip(left_input_bytes.iter().chain(right_input_bytes.iter()))
                .for_each(|(b, l_b)| *b = *l_b);
    
            let base_type_value = U384::from_bytes_be(&buffer).unwrap();
            let new_input_value = PedersenTestField::from_base_type(base_type_value);
            let new_input = TestFieldElement::from(&new_input_value);
    
            self.hash_one(new_input)
        }
    }

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

        let _pederse_p0 = TestCurve::create_point_from_affine(
            TestFieldElement::new(U384::from("49EE3EBA8C1600700EE1B87EB599F16716B0B1022947733551FDE4050CA6804")),
            TestFieldElement::new(U384::from("3CA0CFE4B3BC6DDF346D49D06EA0ED34E621062C0E056C1D0405D266E10268A"))
        ).unwrap();

        let pederse_p1 = TestCurve::create_point_from_affine(
            TestFieldElement::new(U384::from("234287DCBAFFE7F969C748655FCA9E58FA8120B6D56EB0C1080D17957EBE47B")),
            TestFieldElement::new(U384::from("3B056F100F96FB21E889527D41F4E39940135DD7A6C94CC6ED0268EE89E5615")),
        ).unwrap();

        let pederse_p2 = TestCurve::create_point_from_affine(
            TestFieldElement::new(U384::from("4FA56F376C83DB33F9DAB2656558F3399099EC1DE5E3018B7A6932DBA8AA378")),
            TestFieldElement::new(U384::from("3FA0984C931C9E38113E0C0E47E4401562761F92A7A23B45168F4E80FF5B54D")),
        ).unwrap();

        let pederse_p3 = TestCurve::create_point_from_affine(
            TestFieldElement::new(U384::from("4BA4CC166BE8DEC764910F75B45F74B40C690C74709E90F3AA372F0BD2D6997")),
            TestFieldElement::new(U384::from("40301CF5C1751F4B971E46C4EDE85FCAC5C59A5CE5AE7C48151F27B24B219C")),
        ).unwrap();

        let pederse_p4 = TestCurve::create_point_from_affine(
            TestFieldElement::new(U384::from("54302DCB0E6CC1C6E44CCA8F61A63BB2CA65048D53FB325D36FF12C49A58202")),
            TestFieldElement::new(U384::from("1B77B3E37D13504B348046268D8AE25CE98AD783C25561A879DCC77E99C2426")),
        ).unwrap();


        let in1 = FE::new_base("03d937c035c878245caf64531a5756109c53068da139362728feb561405371cb");
        let in2 = FE::new_base("0208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a");
        let expected_hash = FE::new_base("030e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662");

        let hasher = Pedersen::new();

        assert_eq!(hasher.hash_two(in1, in2), expected_hash);

        let parameters = vec![ pederse_p1, pederse_p2, pederse_p3, pederse_p4];
        let parameters = parameters.into_iter().map(|x| x.coordinates().to_vec()).collect();
        let hasher: Pedersen<TestCurve> = Pedersen::from_parameters(parameters);

       // assert_eq!(hasher.hash_two(in1, in2), expected_hash);
    }
}
