#[cfg(test)]
use crate::polynomial::Polynomial;
#[cfg(test)]
use proptest::{collection, prelude::any, prop_compose, strategy::Strategy};

use crate::field::{
    element::FieldElement,
    traits::{IsField, IsTwoAdicField},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U32Field<const MODULUS: u32>;

impl<const MODULUS: u32> IsField for U32Field<MODULUS> {
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

// 15 * 2^27 + 1;
pub type U32TestField = U32Field<2013265921>;
pub type U32TestFieldElement = FieldElement<U32TestField>;

// These params correspond to the 2013265921 modulus.
impl IsTwoAdicField for U32TestField {
    const TWO_ADICITY: u64 = 27;
    const TWO_ADIC_PRIMITVE_ROOT_OF_UNITY: u32 = 440532289;
}

// Functions with different Strategies to use in proptests for FFT in this Field.
#[cfg(test)]
impl U32TestField {
    prop_compose! {
        fn powers_of_two(max_exp: u8)(exp in 1..max_exp) -> usize { 1 << exp }
        // max_exp cannot be multiple of the bits that represent a usize, generally 64 or 32.
        // also it can't exceed the test field's two-adicity.
    }
    prop_compose! {
        fn field_element()(num in any::<u64>().prop_filter("Avoid null coefficients", |x| x != &0)) -> U32TestFieldElement {
            U32TestFieldElement::from(num)
        }
    }
    prop_compose! {
        fn offset()(num in 1..U32TestField::neg(&1)) -> U32TestFieldElement { U32TestFieldElement::from(num as u64) }
    }
    prop_compose! {
        fn vector_with_field_elements(max_exp: u8)(vec in collection::vec(Self::field_element(), 2..1<<max_exp).prop_filter("Avoid polynomials of size not power of two", |vec| vec.len().is_power_of_two())) -> Vec<U32TestFieldElement> {
            vec
        }
    }
    prop_compose! {
        fn poly_with_field_elements(max_exp: u8)(coeffs in Self::vector_with_field_elements(max_exp)) -> Polynomial<U32TestFieldElement> {
            Polynomial::new(&coeffs)
        }
    }
    prop_compose! {
        fn non_power_of_two_sized_field_vec(max_exp: u8)(elem in Self::field_element(), size in Self::powers_of_two(max_exp)) -> Vec<U32TestFieldElement> {
            vec![elem; size + 1]
        }
    }
    prop_compose! {
        fn poly_with_non_power_of_two_coeffs(max_exp: u8)(coeffs in Self::non_power_of_two_sized_field_vec(max_exp)) -> Polynomial<U32TestFieldElement> {
            Polynomial::new(&coeffs)
        }
    }
}
