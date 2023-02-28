pub mod air;
pub mod fri;

use std::primitive;

use air::polynomials::get_cp_and_tp;
use fri::fri_decommit::{fri_decommit_layers, FriDecommitment};
use lambdaworks_crypto::fiat_shamir::transcript::Transcript;
use lambdaworks_crypto::merkle_tree::proof::Proof;
use lambdaworks_math::{
    polynomial::{self, Polynomial},
    traits::ByteConversion,
};
use thiserror::__private::AsDynError;
use winterfell::{
    crypto::hashers::Blake3_256,
    math::{fields::f128::BaseElement, StarkField},
    prover::constraints::CompositionPoly,
    Air, AuxTraceRandElements, Serializable, Trace, TraceTable,
};

use lambdaworks_math::field::element::FieldElement;
use lambdaworks_math::{
    field::fields::u384_prime_field::{IsMontgomeryConfiguration, MontgomeryBackendPrimeField},
    unsigned_integer::element::U384,
};

type U384PrimeField = MontgomeryBackendPrimeField<crate::air::polynomials::MontgomeryConfig>;

type U384FieldElement = FieldElement<U384PrimeField>;

// const MODULUS_MINUS_1: U384 = U384::from("10");

const MODULUS_MINUS_1: U384 = U384::sub(
    &crate::air::polynomials::MontgomeryConfig::MODULUS,
    &U384::from("1"),
)
.0;

const ORDER_OF_ROOTS_OF_UNITY_TRACE: u64 = 4;
const ORDER_OF_ROOTS_OF_UNITY_FOR_LDE: u64 = 16;

pub fn generate_vec_roots(
    subgroup_size: u64,
    coset_factor: u64,
) -> (Vec<U384FieldElement>, U384FieldElement) {
    let MODULUS_MINUS_1_FIELD: U384FieldElement = U384FieldElement::new(MODULUS_MINUS_1);
    let subgroup_size_u384: U384FieldElement = subgroup_size.into();

    let generator_field: U384FieldElement = 3.into();
    let coset_factor_u384: U384FieldElement = coset_factor.into();

    let exp = (&MODULUS_MINUS_1_FIELD) / &subgroup_size_u384;

    let exp_384 = *exp.value();

    let generator_of_subgroup = generator_field.pow(exp_384);

    let mut numbers = Vec::new();

    for exp in 0..subgroup_size {
        let ret = generator_of_subgroup.pow(exp) * &coset_factor_u384;
        numbers.push(ret.clone());
    }

    (numbers, generator_of_subgroup)
}

#[derive(Debug, Clone)]
pub struct StarkQueryProof {
    // TODO: fill this when we know what a proof entails
    pub trace_lde_poly_root: U384FieldElement,
    pub trace_lde_poly_evaluations: Vec<U384FieldElement>,
    /// Merkle paths for the trace polynomial evaluations
    pub trace_lde_poly_inclusion_proofs: Vec<Proof<U384PrimeField, DefaultHasher>>,
    pub composition_poly_lde_evaluations: Vec<U384FieldElement>,

    // composition_poly_root: U384FieldElement,
    pub fri_layers_merkle_roots: Vec<U384FieldElement>,
    // pub fri_layers_merkle_proofs: Vec<(
    //     Proof<U384PrimeField, DefaultHasher>,
    //     Proof<U384PrimeField, DefaultHasher>,
    // )>,
    // pub last_fri_layer_evaluation: U384FieldElement,
    pub fri_decommitment: FriDecommitment,
}

pub type StarkProof = Vec<StarkQueryProof>;

pub use lambdaworks_crypto::merkle_tree::merkle::MerkleTree;
pub use lambdaworks_crypto::merkle_tree::DefaultHasher;
pub type FriMerkleTree = MerkleTree<U384PrimeField, DefaultHasher>;

pub fn fibonacci_trace(initial_values: &[U384FieldElement; 2]) -> Vec<U384FieldElement> {
    let mut ret: Vec<U384FieldElement> = vec![];

    ret.push(initial_values[0].clone());
    ret.push(initial_values[1].clone());

    for i in 2..(ORDER_OF_ROOTS_OF_UNITY_TRACE as usize) {
        ret.push(ret[i - 1].clone() + ret[i - 2].clone());
    }

    ret
}

pub fn prove(
    // air: A,
    // trace: TraceTable<A::BaseField>,
    // pub_inputs: A::PublicInputs,
    pub_inputs: &[U384FieldElement; 2],
) -> StarkQueryProof
// where
//     A: Air<BaseField = BaseElement>,
{
    let transcript = &mut Transcript::new();
    let pub_inputs_bytes: Vec<_> = pub_inputs
        .iter()
        .map(|value| value.to_bytes_be())
        .flatten()
        .collect();
    transcript.append(&pub_inputs_bytes);
    // * Generate composition polynomials using Winterfell
    // let (mut composition_poly, mut trace_poly) = get_cp_and_tp(air, trace, pub_inputs).unwrap();

    // * Generate Coset
    let (roots_of_unity, primitive_root) =
        crate::generate_vec_roots(ORDER_OF_ROOTS_OF_UNITY_TRACE, 1);

    let (lde_roots_of_unity, lde_primitive_root) =
        crate::generate_vec_roots(ORDER_OF_ROOTS_OF_UNITY_FOR_LDE, 1);

    let trace = fibonacci_trace(pub_inputs);

    let mut trace_poly = Polynomial::interpolate(&roots_of_unity, &trace);

    // TODO: For the composition polynomial, we should sample numbers alpha_1, ..., alpha_k through fiat shamir
    // to get a linear combination of all the transition polynomials.
    let mut composition_poly = get_composition_poly(trace_poly.clone(), &primitive_root);

    // * Do Reed-Solomon on the trace and composition polynomials using some blowup factor
    let composition_poly_lde = composition_poly.evaluate_slice(lde_roots_of_unity.as_slice());
    let trace_poly_lde = trace_poly.evaluate_slice(lde_roots_of_unity.as_slice());

    // * Commit to both polynomials using a Merkle Tree
    let composition_poly_lde_merkle_tree = FriMerkleTree::build(composition_poly_lde.as_slice());
    let trace_poly_lde_merkle_tree = FriMerkleTree::build(&trace_poly_lde.as_slice());

    // * Do FRI on the composition polynomials

    let mut composition_poly: Polynomial<U384FieldElement> = Polynomial {
        coefficients: [
            U384FieldElement::one(),
            U384FieldElement::zero(),
            U384FieldElement::one(),
            U384FieldElement::one(),
        ]
        .to_vec(),
    };

    /*
        IMPORTANT NOTE:
        When we commit to the trace polynomial, let's call it f, we commit to an LDE of it.
        On the other hand, the fibonacci constraint (and in general, any constraint) related to f applies
        only using non-LDE roots of unity.
        In this case, the constraint is f(w^2 x) - f(w x) - f(x), where w is a 2^n root of unity.
        But for the commitment we use g, a 2^{nb} root of unity (b is the blowup factor).
        When we sample a value x to evaluate the trace polynomial on, it has to be a 2^{nb} root of unity,
        so with fiat-shamir we sample a random index in that range.
        When we provide evaluations, we provide them for x*(w^2), x*w and x.
    */

    // TODO: These should be q_1, ..., q_m
    let q_1: usize = usize::from_be_bytes(transcript.challenge()[0..8].try_into().unwrap())
        % ORDER_OF_ROOTS_OF_UNITY_FOR_LDE as usize;

    let evaluation_points = vec![
        lde_primitive_root.pow(q_1),
        lde_primitive_root.pow(q_1) * &primitive_root,
        lde_primitive_root.pow(q_1) * (&primitive_root * &primitive_root),
    ];

    let trace_lde_poly_evaluations = trace_poly.evaluate_slice(&evaluation_points);
    let composition_poly_lde_evaluation = composition_poly.evaluate(&evaluation_points[0]);

    let mut merkle_paths = vec![];

    merkle_paths.push(
        trace_poly_lde_merkle_tree
            .get_proof_by_pos(q_1, trace_lde_poly_evaluations[0].clone())
            .unwrap(),
    );
    merkle_paths.push(
        trace_poly_lde_merkle_tree
            .get_proof_by_pos(
                q_1 + (ORDER_OF_ROOTS_OF_UNITY_FOR_LDE / ORDER_OF_ROOTS_OF_UNITY_TRACE) as usize,
                trace_lde_poly_evaluations[1].clone(),
            )
            .unwrap(),
    );
    merkle_paths.push(
        trace_poly_lde_merkle_tree
            .get_proof_by_pos(
                q_1 + (ORDER_OF_ROOTS_OF_UNITY_FOR_LDE / ORDER_OF_ROOTS_OF_UNITY_TRACE) as usize
                    * 2,
                trace_lde_poly_evaluations[2].clone(),
            )
            .unwrap(),
    );

    let trace_root = trace_poly_lde_merkle_tree.root.clone();

    // * Sample q_1, ..., q_m using Fiat-Shamir
    let decommitment_index: usize =
        usize::from_be_bytes(transcript.challenge()[0..8].try_into().unwrap())
            % ORDER_OF_ROOTS_OF_UNITY_FOR_LDE as usize;

    // BEGIN FRI
    let lde_fri_commitment =
        crate::fri::fri(&mut composition_poly, &lde_roots_of_unity, transcript);

    // * For every q_i, do FRI decommitment
    let fri_decommitment = fri_decommit_layers(&lde_fri_commitment, decommitment_index);

    let fri_layers_merkle_roots: Vec<U384FieldElement> = lde_fri_commitment
        .iter()
        .map(|fri_commitment| fri_commitment.merkle_tree.root.clone())
        .collect();

    // END FRI

    StarkQueryProof {
        trace_lde_poly_root: trace_root,
        trace_lde_poly_evaluations,
        trace_lde_poly_inclusion_proofs: merkle_paths,
        composition_poly_lde_evaluations: vec![composition_poly_lde_evaluation],
        fri_layers_merkle_roots: fri_layers_merkle_roots,
        fri_decommitment: fri_decommitment,
    }
}

fn get_composition_poly(
    trace_poly: Polynomial<U384FieldElement>,
    root_of_unity: &U384FieldElement,
) -> Polynomial<U384FieldElement> {
    let w_squared_x = Polynomial::new(&vec![
        U384FieldElement::zero(),
        root_of_unity * root_of_unity,
    ]);
    let w_x = Polynomial::new(&vec![U384FieldElement::zero(), root_of_unity.clone()]);

    polynomial::compose(&trace_poly, &w_squared_x)
        - polynomial::compose(&trace_poly, &w_x)
        - trace_poly
}

pub fn verify(pub_inputs: &[U384FieldElement; 2], proof: StarkQueryProof) -> bool {
    let transcript = &mut Transcript::new();
    let pub_inputs_bytes: Vec<_> = pub_inputs
        .iter()
        .map(|value| value.to_bytes_be())
        .flatten()
        .collect();
    transcript.append(&pub_inputs_bytes);

    let trace_poly_root = proof.trace_lde_poly_root;

    // TODO: For the composition polynomial, we should sample numbers alpha_1, ..., alpha_k through fiat shamir
    // to get a linear combination of all the transition polynomials.

    let (_roots_of_unity, mut primitive_root) =
        crate::generate_vec_roots(ORDER_OF_ROOTS_OF_UNITY_FOR_LDE, 1);
    let evaluations = proof.trace_lde_poly_evaluations;

    // TODO: These could be multiple evaluations depending on how many q_i are sampled with Fiat Shamir
    let composition_poly_lde_evaluation = proof.composition_poly_lde_evaluations[0].clone();

    for merkle_proof in proof.trace_lde_poly_inclusion_proofs {
        if !merkle_proof.verify(trace_poly_root.clone()) {
            return false;
        }
    }

    // We don't care about the value of this challenge (as it's the index of the evaluation point in the merkle tree),
    // but we still need to challenge to advance the transcript and generate the same values as the prover.
    transcript.challenge();

    /*
    if composition_poly_lde_evaluation != &evaluations[2] - &evaluations[1] - &evaluations[0] {
        return false;
    } */

    for merkle_root in proof.fri_layers_merkle_roots.clone() {
        transcript.append(&merkle_root.value().to_bytes_be());
    }

    transcript.append(
        &proof
            .fri_decommitment
            .last_layer_evaluation
            .value()
            .to_bytes_be(),
    );

    // FRI VERIFYING BEGINS HERE
    let decommitment_index: usize =
        usize::from_be_bytes(transcript.challenge()[0..8].try_into().unwrap())
            % ORDER_OF_ROOTS_OF_UNITY_FOR_LDE as usize;

    for (
        layer_number,
        (fri_layer_merkle_root, (fri_layer_auth_path, fri_layer_auth_path_symmetric)),
    ) in proof
        .fri_layers_merkle_roots
        .iter()
        .zip(proof.fri_decommitment.layer_merkle_paths.iter())
        .enumerate()
        .skip(1)
    {
        if !fri_layer_auth_path.verify(fri_layer_merkle_root.clone()) {
            return false;
        }

        if !fri_layer_auth_path_symmetric.verify(fri_layer_merkle_root.clone()) {
            return false;
        }

        // TODO: use Fiat Shamir
        let beta: u64 = 3;

        let (previous_auth_path, previous_auth_path_symmetric) = match proof
            .fri_decommitment
            .layer_merkle_paths
            .get(layer_number - 1)
        {
            Some(previous) => previous,
            None => return false,
        };

        let evaluation_point = primitive_root.pow(decommitment_index);

        let v = (previous_auth_path.clone().value + previous_auth_path_symmetric.clone().value)
            / U384FieldElement::new(U384::from("2"))
            + U384FieldElement::new(U384::from_u64(beta))
                * (previous_auth_path.clone().value - previous_auth_path_symmetric.clone().value)
                / (U384FieldElement::new(U384::from("2")) * evaluation_point);

        primitive_root = primitive_root.pow(2_usize);

        if v != fri_layer_auth_path.value {
            return false;
        }
    }

    // For each fri layer merkle proof check:
    // That each merkle path verifies

    // Sample beta with fiat shamir
    // Compute v = [P_i(z_i) + P_i(-z_i)] / 2 + beta * [P_i(z_i) - P_i(-z_i)] / (2 * z_i)
    // Where P_i is the folded polynomial of the i-th fiat shamir round
    // z_i is obtained from the first z (that was derived through fiat-shamir) through a known calculation
    // The calculation is, given the index, index % length_of_evaluation_domain

    // Check that v = P_{i+1}(z_i)

    return true;
}

// TODOS after basic fibonacci works:
// - Add Fiat Shamir
// - Add Zerofiers
// - Instead of returning a bool, make an error type encoding each possible failure in the verifying pipeline so failures give more info.
// - Unhardcode polynomials, use Winterfell AIR
// - Coset evaluation

#[cfg(test)]
mod tests {
    use crate::{verify, U384FieldElement};

    use super::prove;
    use lambdaworks_math::{field::element::FieldElement, unsigned_integer::element::U384};
    use winterfell::{FieldExtension, ProofOptions};

    #[test]
    fn test_prove() {
        let pub_inputs = [
            U384FieldElement::new(U384::from("1")),
            U384FieldElement::new(U384::from("1")),
        ];
        let result = prove(&pub_inputs);
        assert!(verify(&pub_inputs, result));
    }
}
