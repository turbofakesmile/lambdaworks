use crate::field::traits::{IsField, IsTwoAdicField};

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct U32TestField<const MODULUS: u32>;

impl<const MODULUS: u32> IsField for U32TestField<MODULUS> {
    type BaseType = u32;

    fn add(a: &u32, b: &u32) -> u32 {
        ((*a as u128 + *b as u128) % MODULUS as u128) as u32
    }

    fn sub(a: &u32, b: &u32) -> u32 {
        (((*a as u128 + MODULUS as u128) - *b as u128) % MODULUS as u128) as u32
    }

    fn neg(a: &u32) -> u32 {
        MODULUS - a
    }

    fn mul(a: &u32, b: &u32) -> u32 {
        ((*a as u128 * *b as u128) % MODULUS as u128) as u32
    }

    fn div(a: &u32, b: &u32) -> u32 {
        Self::mul(a, &Self::inv(b))
    }

    fn inv(a: &u32) -> u32 {
        assert_ne!(*a, 0, "Cannot invert zero element");
        Self::pow(a, MODULUS - 2)
    }

    fn eq(a: &u32, b: &u32) -> bool {
        Self::from_base_type(*a) == Self::from_base_type(*b)
    }

    fn zero() -> u32 {
        0
    }

    fn one() -> u32 {
        1
    }

    fn from_u64(x: u64) -> u32 {
        (x % MODULUS as u64) as u32
    }

    fn from_base_type(x: u32) -> u32 {
        x % MODULUS
    }
}

pub type U32TwoAdicTestField = U32TestField<2013265921>;

impl IsTwoAdicField for U32TwoAdicTestField {
    const TWO_ADICITY: u64 = 27;
    const TWO_ADIC_PRIMITVE_ROOT_OF_UNITY: u32 = 440564289;
}
