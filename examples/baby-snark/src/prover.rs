use crate::{
    common::*,
    serialization::{deserialize_value, serialize_value},
    setup::ProvingKey,
    ssp::SquareSpanProgram,
};
use lambdaworks_math::{errors::DeserializationError, msm::pippenger::msm};

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
            serialize_value(&self.h),
            serialize_value(&self.v_w),
            serialize_value(&self.v_w_prime),
            serialize_value(&self.b_w),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let mut offset = 0;

        let (h, read) = deserialize_value(&bytes[offset..])?;
        offset += read;

        let (v_w, read) = deserialize_value(&bytes[offset..])?;
        offset += read;

        let (v_w_prime, read) = deserialize_value(&bytes[offset..])?;
        offset += read;

        let (b_w, _) = deserialize_value(&bytes[offset..])?;

        Ok(Self {
            h,
            v_w,
            v_w_prime,
            b_w,
        })
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
