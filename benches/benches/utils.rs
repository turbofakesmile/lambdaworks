use ark_ff::{BigInt, MontBackend, Fp256};
use ark_std::UniformRand;
use lambdaworks_math::{
    field::{
        element::FieldElement, fields::montgomery_backed_prime_fields::{IsModulus, U256PrimeField}
    },
    unsigned_integer::element::{UnsignedInteger, U256},
};
use rand::SeedableRng;


#[derive(ark_ff::MontConfig)]
#[modulus = "115792089237316195423570985008687907853269984665640564039457584007913129639747"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp256<MontBackend<FqConfig, 4>>;


#[derive(Clone, Debug, Hash, Copy)]
pub struct MontgomeryConfigStark252PrimeField;
impl IsModulus<U256> for MontgomeryConfigStark252PrimeField {
    const MODULUS: U256 =
        U256::from_hex_unchecked("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff43");
}

pub type Stark252PrimeField = U256PrimeField<MontgomeryConfigStark252PrimeField>;

/// Creates `amount` random elements
pub fn generate_random_elements(amount: u64) -> Vec<Fq> {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(9001);
    let mut arkworks_vec = Vec::new();
    for _i in 0..amount {
        let a = Fq::rand(&mut rng);
        arkworks_vec.push(a);
    }

    arkworks_vec
}

pub fn to_lambdaworks_vec(arkworks_vec: &[Fq]) -> Vec<FieldElement<Stark252PrimeField>> {
    let mut lambdaworks_vec = Vec::new();
    for &arkworks_felt in arkworks_vec {
        let big_int: BigInt<4> = arkworks_felt.into();
        let mut limbs = big_int.0;
        limbs.reverse();

        let a: FieldElement<Stark252PrimeField> = FieldElement::from(&UnsignedInteger { limbs });

        assert_eq!(a.representative().limbs, limbs);

        lambdaworks_vec.push(a);
    }

    lambdaworks_vec
}
