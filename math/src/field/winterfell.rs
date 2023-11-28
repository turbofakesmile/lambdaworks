use crate::{
    field::{element::FieldElement, traits::IsField},
    traits::Serializable,
    unsigned_integer::element::U256,
};
pub use miden_core::Felt;
use miden_core::QuadExtension;
pub use winter_math::fields::f128::BaseElement;
use winter_math::{FieldElement as IsWinterfellFieldElement, StarkField};

use super::traits::{IsFFTField, IsPrimeField};

impl IsFFTField for Felt {
    const TWO_ADICITY: u64 = <Felt as StarkField>::TWO_ADICITY as u64;
    const TWO_ADIC_PRIMITVE_ROOT_OF_UNITY: Self::BaseType = Felt::TWO_ADIC_ROOT_OF_UNITY;
}

impl IsField for Felt {
    type BaseType = Felt;

    fn add(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        *a + *b
    }

    fn mul(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        *a * *b
    }

    fn sub(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        *a - *b
    }

    fn neg(a: &Self::BaseType) -> Self::BaseType {
        -*a
    }

    fn inv(a: &Self::BaseType) -> Result<Self::BaseType, super::errors::FieldError> {
        Ok((*a).inv())
    }

    fn div(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        *a / *b
    }

    fn eq(a: &Self::BaseType, b: &Self::BaseType) -> bool {
        *a == *b
    }

    fn zero() -> Self::BaseType {
        Self::BaseType::ZERO
    }

    fn one() -> Self::BaseType {
        Self::BaseType::ONE
    }

    fn from_u64(x: u64) -> Self::BaseType {
        Self::BaseType::from(x)
    }

    fn from_base_type(x: Self::BaseType) -> Self::BaseType {
        x
    }
}

impl Serializable for FieldElement<Felt> {
    fn serialize(&self) -> Vec<u8> {
        Felt::elements_as_bytes(&[*self.value()]).to_vec()
    }
}


impl IsFFTField for QuadExtension<Felt> {
    const TWO_ADICITY: u64 = <Felt as IsFFTField>::TWO_ADICITY;
    const TWO_ADIC_PRIMITVE_ROOT_OF_UNITY: Self::BaseType = QuadExtension::new(Felt::TWO_ADIC_PRIMITVE_ROOT_OF_UNITY, Felt::ZERO);
}

impl IsField for QuadExtension<Felt> {
    type BaseType = QuadExtension<Felt>;

    fn add(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        todo!()
    }

    fn mul(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        todo!()
    }

    fn sub(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        todo!()
    }

    fn neg(a: &Self::BaseType) -> Self::BaseType {
        todo!()
    }

    fn inv(a: &Self::BaseType) -> Result<Self::BaseType, super::errors::FieldError> {
        todo!()
    }

    fn div(a: &Self::BaseType, b: &Self::BaseType) -> Self::BaseType {
        todo!()
    }

    fn eq(a: &Self::BaseType, b: &Self::BaseType) -> bool {
        todo!()
    }

    fn zero() -> Self::BaseType {
        todo!()
    }

    fn one() -> Self::BaseType {
        todo!()
    }

    fn from_u64(x: u64) -> Self::BaseType {
        todo!()
    }

    fn from_base_type(x: Self::BaseType) -> Self::BaseType {
        todo!()
    }
}

impl Serializable for FieldElement<QuadExtension<Felt>> {

}

