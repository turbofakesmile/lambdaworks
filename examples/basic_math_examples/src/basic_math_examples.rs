use lambdaworks_math::{
    field::{element::FieldElement, traits::IsField},
    polynomial::Polynomial,
    IsEllipticCurve,
};

use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::random;

type Ed25519MontgomeryBackendPrimeField<T> = MontgomeryBackendPrimeField<T, 4>;
// This will create the prime field corresponding to the modulus 2^255 - 19

pub struct MontgomeryConfigEd25519PrimeField;
impl IsModulus<U256> for MontgomeryConfigEd25519PrimeField {
    const MODULUS: U256 = U256::from_hex_unchecked(
        "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFED",
    );
}

pub type Ed25519PrimeField = Ed25519MontgomeryBackendPrimeField<MontgomeryConfigEd25519PrimeField>;

type FE = Ed25519PrimeField; 

pub struct Ed25519Curve;

impl IsEllipticCurve for Ed25519Curve {
    type BaseField = Ed25519PrimeField;
    type PointRepresentation = EdwardsProjectivePoint<Self>;

    fn generator() -> Self::PointRepresentation {
        Self::PointRepresentation::new([
            FieldElement::<Self::BaseField>::from_hex_unchecked("0x216936D3CD6E53FEC0A4E231FDD6DC5C692CC7609525A7B2C9562D608F25D51A"),
            FieldElement::<Self::BaseField>::from_hex_unchecked("0x6666666666666666666666666666666666666666666666666666666666666658"),
            FieldElement::one(),
        ])
    }
}

impl IsEdwards for Ed22519Curve {
    fn a() -> FieldElement<Self::BaseField> {
        FieldElement::from_hex("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffec");
    }

    fn d() -> FieldElement<Self::BaseField> {
        -FieldElement::from_hex("0x52036cee2b6ffe738cc740797779e89800700a4d4141d8ab75eb4dca135978a3");
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {

    use crate::shamir_secret_sharing::ShamirSecretSharing;
    use lambdaworks_math::field::element::FieldElement;
    use lambdaworks_math::field::fields::u64_prime_field::U64PrimeField;

    #[test]
    fn check_sum() {
        let a = Ed25519PrimeField::from_hex("3").unwrap();
        let b = Ed25519PrimeField::from_hex("5").unwrap();
        let c = FE::from_hex("8").unwrap;
        assert_eq!(&a + &b, c);
    }
}
