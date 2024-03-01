use std::mem::size_of;

use crate::{common::*, setup::ProvingKey, ssp::SquareSpanProgram};
use lambdaworks_math::{
    errors::DeserializationError,
    msm::pippenger::msm,
    traits::{AsBytes, Deserializable},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Proof {
    pub h: G1Point,
    pub v_w: G1Point,
    pub v_w_prime: G2Point,
    pub b_w: G1Point,
}

impl Proof {
    pub fn serialize(&self) -> Vec<u8> {
        [
            Self::serialize_commitment(&self.h),
            Self::serialize_commitment(&self.v_w),
            Self::serialize_commitment(&self.v_w_prime),
            Self::serialize_commitment(&self.b_w),
        ]
        .iter()
        .fold(Vec::new(), |mut bytes, serialized| {
            bytes.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
            bytes.extend_from_slice(serialized);

            bytes
        })
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let (offset, h) = Self::deserialize_commitment::<G1Point>(bytes, 0)?;
        let (offset, v_w) = Self::deserialize_commitment::<G1Point>(bytes, offset)?;
        let (offset, v_w_prime) = Self::deserialize_commitment::<G2Point>(bytes, offset)?;
        let (_, b_w) = Self::deserialize_commitment::<G1Point>(bytes, offset)?;

        Ok(Self {
            h,
            v_w,
            v_w_prime,
            b_w,
        })
    }

    fn serialize_commitment<Commitment: AsBytes>(cm: &Commitment) -> Vec<u8> {
        cm.as_bytes()
    }

    fn deserialize_commitment<Commitment: Deserializable>(
        bytes: &[u8],
        mut offset: usize,
    ) -> Result<(usize, Commitment), DeserializationError> {
        let element_size_bytes: [u8; size_of::<u32>()] = bytes
            .get(offset..offset + size_of::<u32>())
            .ok_or(DeserializationError::InvalidAmountOfBytes)?
            .try_into()
            .map_err(|_| DeserializationError::InvalidAmountOfBytes)?;
        let element_size = u32::from_be_bytes(element_size_bytes) as usize;
        offset += size_of::<u32>();

        let commitment = Commitment::deserialize(
            bytes
                .get(offset..offset + element_size)
                .ok_or(DeserializationError::InvalidAmountOfBytes)?,
        )?;
        offset += element_size;

        Ok((offset, commitment))
    }
}

pub struct Prover;
impl Prover {
    pub fn prove(inputs: &[FrElement], ssp: &SquareSpanProgram, pk: &ProvingKey) -> Proof {
        let h_coefficients = ssp
            .calculate_h_coefficients(inputs)
            .iter()
            .map(|elem| elem.representative())
            .collect::<Vec<_>>();

        let h = msm(&h_coefficients, &pk.k_powers_of_tau_g1).unwrap();
        let w = inputs
            .iter()
            .skip(ssp.number_of_public_inputs)
            .map(|elem| elem.representative())
            .collect::<Vec<_>>();

        let v_w = msm(&w, &pk.u_tau_g1).unwrap();

        let v_w_prime = msm(&w, &pk.u_tau_g2).unwrap();

        let b_w = msm(&w, &pk.beta_u_tau_g1).unwrap();

        Proof {
            h,
            v_w,
            v_w_prime,
            b_w,
        }
    }
}

#[cfg(test)]
mod tests {
    use lambdaworks_math::{cyclic_group::IsGroup, elliptic_curve::traits::IsEllipticCurve};

    use super::*;

    #[test]
    fn can_serialize_proof() {
        let proof = Proof {
            h: Curve::generator().operate_with_self(1usize),
            v_w: Curve::generator().operate_with_self(2usize),
            v_w_prime: TwistedCurve::generator().operate_with_self(3usize),
            b_w: Curve::generator().operate_with_self(4usize),
        };

        let serialized = proof.serialize();

        let deserialized = Proof::deserialize(&serialized).unwrap();

        assert_eq!(deserialized, proof);
    }
}
