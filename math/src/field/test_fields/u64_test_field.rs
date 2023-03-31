#[cfg(test)]
use crate::polynomial::Polynomial;
#[cfg(test)]
use proptest::{collection, prelude::any, prop_compose, strategy::Strategy};

use crate::field::{
    element::FieldElement,
    traits::{IsField, IsPrimeField, IsTwoAdicField},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U64Field<const MODULUS: u64>;

impl<const MODULUS: u64> IsField for U64Field<MODULUS> {
    type BaseType = u64;

    fn add(a: &u64, b: &u64) -> u64 {
        ((*a as u128 + *b as u128) % MODULUS as u128) as u64
    }

    fn sub(a: &u64, b: &u64) -> u64 {
        (((*a as u128 + MODULUS as u128) - *b as u128) % MODULUS as u128) as u64
    }

    fn neg(a: &u64) -> u64 {
        MODULUS - a
    }

    fn mul(a: &u64, b: &u64) -> u64 {
        ((*a as u128 * *b as u128) % MODULUS as u128) as u64
    }

    fn div(a: &u64, b: &u64) -> u64 {
        Self::mul(a, &Self::inv(b))
    }

    fn inv(a: &u64) -> u64 {
        assert_ne!(*a, 0, "Cannot invert zero element");
        Self::pow(a, MODULUS - 2)
    }

    fn eq(a: &u64, b: &u64) -> bool {
        Self::from_u64(*a) == Self::from_u64(*b)
    }

    fn zero() -> u64 {
        0
    }

    fn one() -> u64 {
        1
    }

    fn from_u64(x: u64) -> u64 {
        x % MODULUS
    }

    fn from_base_type(x: u64) -> u64 {
        Self::from_u64(x)
    }
}

impl<const MODULUS: u64> IsPrimeField for U64Field<MODULUS> {
    type RepresentativeType = u64;

    fn representative(x: &u64) -> u64 {
        *x
    }
}

pub type U64TestField = U64Field<18446744069414584321>;
pub type U64TestFieldElement = FieldElement<U64TestField>;

// These params correspond to the 18446744069414584321 modulus.
impl IsTwoAdicField for U64TestField {
    const TWO_ADICITY: u64 = 32;
    const TWO_ADIC_PRIMITVE_ROOT_OF_UNITY: u64 = 1753635133440165772;
}

// This implements all the structures used in the proptests of FFT in this field.
#[cfg(test)]
impl U64TestField {
    prop_compose! {
        pub fn powers_of_two(max_exp: u8)(exp in 1..max_exp) -> usize { 1 << exp }
        // max_exp cannot be multiple of the bits that represent a usize, generally 64 or 32.
        // also it can't exceed the test field's two-adicity.
    }
    prop_compose! {
        pub fn field_element()(num in any::<u64>().prop_filter("Avoid null coefficients", |x| x != &0)) -> U64TestFieldElement {
            U64TestFieldElement::from(num)
        }
    }
    prop_compose! {
        pub fn offset()(num in 1..U64TestField::neg(&1)) -> U64TestFieldElement { U64TestFieldElement::from(num) }
    }
    prop_compose! {
        pub fn vector_with_field_elements(max_exp: u8)(vec in collection::vec(Self::field_element(), 2..1<<max_exp).prop_filter("Avoid polynomials of size not power of two", |vec| vec.len().is_power_of_two())) -> Vec<U64TestFieldElement> {
            vec
        }
    }
    prop_compose! {
        pub fn poly_with_field_elements(max_exp: u8)(coeffs in Self::vector_with_field_elements(max_exp)) -> Polynomial<U64TestFieldElement> {
            Polynomial::new(&coeffs)
        }
    }
    prop_compose! {
        pub fn non_power_of_two_sized_field_vec(max_exp: u8)(elem in Self::field_element(), size in Self::powers_of_two(max_exp)) -> Vec<U64TestFieldElement> {
            vec![elem; size + 1]
        }
    }
    prop_compose! {
        pub fn poly_with_non_power_of_two_coeffs(max_exp: u8)(coeffs in Self::non_power_of_two_sized_field_vec(max_exp)) -> Polynomial<U64TestFieldElement> {
            Polynomial::new(&coeffs)
        }
    }
}
