use lambdaworks_math::{field::{element::FieldElement, fields::montgomery_backed_prime_fields::{IsModulus, U256PrimeField}}, unsigned_integer::element::U256};


#[derive(Clone, Debug, Hash, Copy)]
struct MontgomeryPoseidonPrimeField;
impl IsModulus<U256> for MontgomeryPoseidonPrimeField {

    // Change the modulus for the one you want to use. This is the Stark252 Field modulus
    const MODULUS: U256 =
        U256::from_hex_unchecked("800000000000011000000000000000000000000000000000000000000000001");
}

/* 
const constants = [
]
*/

type F = U256PrimeField<MontgomeryPoseidonPrimeField>;
type FE = FieldElement<F>;

fn main() {

    println!("Hello world");
}
