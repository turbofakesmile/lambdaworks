use crate::field::element::FieldElement;
use crate::field::fields::u64_prime_field::U64PrimeField;

use crate::field::traits::IsTwoAdicField;


pub type Ecgfp5 = U64PrimeField<0xFFFF_FFFF_0000_0001_u64>;
pub type Ecgfp5FE = FieldElement<Ecgfp5>;



impl IsTwoAdicField for Ecgfp5 {
    const TWO_ADICITY: u64 = 4;
    const TWO_ADIC_PRIMITVE_ROOT_OF_UNITY: u64 = 3;
}
