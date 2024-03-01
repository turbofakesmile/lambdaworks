use std::mem::size_of;

use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::traits::{IsEllipticCurve, IsPairing},
    errors::DeserializationError,
    field::traits::IsField,
    traits::{AsBytes, ByteConversion, Deserializable},
};

use crate::{
    common::{
        sample_fr_elem, Curve, FrElement, G1Point, G2Point, Pairing, PairingOutput, TwistedCurve,
    },
    serialization::{deserialize_value, deserialize_vec, serialize_value, serialize_vec},
    ssp::SquareSpanProgram,
};

#[derive(Debug, PartialEq, Eq)]
pub struct VerifyingKey {
    // Ui(τ) * g1, 0 <= i < l
    pub u_tau_g1: Vec<G1Point>,
    // Ui(τ) * g2, 0 <= i < l
    pub u_tau_g2: Vec<G2Point>,
    // t(τ) * g2
    pub t_tau_g2: G2Point,
    // e(g1, g2)^-1
    pub inv_pairing_g1_g2: PairingOutput,
    // β * γ * g1
    pub beta_gamma_g1: G1Point,
    // γ * g2
    pub gamma_g2: G2Point,
}

pub struct ProvingKey {
    // (τ^k) * g1, 0 <= k < m
    pub k_powers_of_tau_g1: Vec<G1Point>,
    // Ui(τ) * g1, l <= i <= m
    pub u_tau_g1: Vec<G1Point>,
    // Ui(τ) * g2, l <= i <= m
    pub u_tau_g2: Vec<G2Point>,
    // β * Ui(τ) * g1, l <= i <= m
    pub beta_u_tau_g1: Vec<G1Point>,
}

struct ToxicWaste {
    tau: FrElement,
    beta: FrElement,
    gamma: FrElement,
}

impl ToxicWaste {
    pub fn new() -> Self {
        Self {
            tau: sample_fr_elem(),
            beta: sample_fr_elem(),
            gamma: sample_fr_elem(),
        }
    }
}

pub fn setup(u: &SquareSpanProgram) -> (ProvingKey, VerifyingKey) {
    let g1: G1Point = Curve::generator();
    let g2: G2Point = TwistedCurve::generator();

    let tw = ToxicWaste::new();

    let u_tau = u.u_polynomials.iter().map(|p| p.evaluate(&tw.tau));

    let vk = VerifyingKey {
        u_tau_g1: u_tau
            .clone()
            .take(u.number_of_public_inputs)
            .map(|ui| g1.operate_with_self(ui.representative()))
            .collect(),
        u_tau_g2: u_tau
            .clone()
            .take(u.number_of_public_inputs)
            .map(|ui| g2.operate_with_self(ui.representative()))
            .collect(),
        t_tau_g2: g2.operate_with_self(
            (tw.tau.pow(u.number_of_constraints) - FrElement::one()).representative(),
        ),
        inv_pairing_g1_g2: Pairing::compute(&g1, &g2).unwrap().inv().unwrap(),
        beta_gamma_g1: g1.operate_with_self((&tw.beta * &tw.gamma).representative()),
        gamma_g2: g2.operate_with_self(tw.gamma.representative()),
    };

    let pk = ProvingKey {
        k_powers_of_tau_g1: (0..u.number_of_constraints - 1)
            .map(|k| g1.operate_with_self(tw.tau.pow(k).representative()))
            .collect(),
        u_tau_g1: u_tau
            .clone()
            .take(u.number_of_constraints + 1)
            .skip(u.number_of_public_inputs)
            .map(|ui| g1.operate_with_self(ui.representative()))
            .collect(),
        u_tau_g2: u_tau
            .clone()
            .take(u.number_of_constraints + 1)
            .skip(u.number_of_public_inputs)
            .map(|ui| g2.operate_with_self(ui.representative()))
            .collect(),
        beta_u_tau_g1: u_tau
            .clone()
            .take(u.number_of_constraints + 1)
            .skip(u.number_of_public_inputs)
            .map(|ui| g1.operate_with_self((ui * (&tw.beta)).representative()))
            .collect(),
    };

    (pk, vk)
}

impl VerifyingKey {
    pub fn serialize(&self) -> Vec<u8> {
        [
            serialize_vec(&self.u_tau_g1),
            serialize_vec(&self.u_tau_g2),
            serialize_value(&self.t_tau_g2),
            // serialize_value(self.inv_pairing_g1_g2),
            serialize_value(&self.beta_gamma_g1),
            serialize_value(&self.gamma_g2),
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

        let (u_tau_g1, read) = deserialize_vec(&bytes[offset..])?;
        offset += read;

        let (u_tau_g2, read) = deserialize_vec(&bytes[offset..])?;
        offset += read;

        // let (inv_pairing_g1_g2, read) = deserialize_value(&bytes[offset..])?;
        // offset += read;

        let (t_tau_g2, read) = deserialize_value(&bytes[offset..])?;
        offset += read;

        let (beta_gamma_g1, read) = deserialize_value(&bytes[offset..])?;
        offset += read;

        let (gamma_g2, _) = deserialize_value(&bytes[offset..])?;

        Ok(Self {
            u_tau_g1,
            u_tau_g2,
            t_tau_g2,
            beta_gamma_g1,
            gamma_g2,
            inv_pairing_g1_g2: PairingOutput::new(
                <<Pairing as IsPairing>::OutputField as IsField>::one(),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::scs::SquareConstraintSystem;

    use super::*;

    fn sample_matrix() -> Vec<Vec<FrElement>> {
        vec![
            vec![FrElement::from(1), FrElement::from(2), FrElement::from(3)],
            vec![FrElement::from(4), FrElement::from(5), FrElement::from(6)],
            vec![FrElement::from(7), FrElement::from(8), FrElement::from(9)],
        ]
    }

    #[test]
    fn can_serialize_verifying_key() {
        let scs = SquareConstraintSystem::from_matrix(sample_matrix(), 2);
        let ssp = SquareSpanProgram::from_scs(scs);
        let (_, mut verifying_key) = setup(&ssp);

        verifying_key.inv_pairing_g1_g2 =
            PairingOutput::new(<<Pairing as IsPairing>::OutputField as IsField>::one());

        let serialized = verifying_key.serialize();
        let mut deserialized = VerifyingKey::deserialize(&serialized).unwrap();

        deserialized.inv_pairing_g1_g2 =
            PairingOutput::new(<<Pairing as IsPairing>::OutputField as IsField>::one());

        assert_eq!(deserialized, verifying_key);
    }
}
