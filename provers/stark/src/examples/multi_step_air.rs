use std::iter;

use lambdaworks_math::field::{
    element::FieldElement, fields::fft_friendly::stark_252_prime_field::Stark252PrimeField,
    traits::IsFFTField,
};

use crate::{
    constraints::boundary::{BoundaryConstraint, BoundaryConstraints},
    context::AirContext,
    frame::Frame,
    proof::options::ProofOptions,
    trace::TraceTable,
    traits::AIR,
    transcript::IsStarkTranscript,
    Felt252,
};

#[derive(Clone)]
pub struct MultiStepAIR {
    context: AirContext,
    trace_length: usize,
}

impl AIR for MultiStepAIR {
    type Field = Stark252PrimeField;
    type RAPChallenges = ();
    type PublicInputs = ();

    const STEP_SIZE: usize = 16;

    fn new(
        trace_length: usize,
        _pub_inputs: &Self::PublicInputs,
        proof_options: &ProofOptions,
    ) -> Self {
        let context = AirContext {
            proof_options: proof_options.clone(),
            trace_columns: 1,
            transition_exemptions: vec![0],
            transition_offsets: vec![0],
            num_transition_constraints: 2,
        };

        Self {
            context,
            trace_length,
        }
    }

    fn build_auxiliary_trace(
        &self,
        _main_trace: &TraceTable<Self::Field>,
        _rap_challenges: &Self::RAPChallenges,
    ) -> TraceTable<Self::Field> {
        TraceTable::empty()
    }

    fn build_rap_challenges(
        &self,
        _transcript: &mut impl IsStarkTranscript<Self::Field>,
    ) -> Self::RAPChallenges {
    }

    fn compute_transition(
        &self,
        frame: &Frame<Self::Field>,
        _periodic_values: &[FieldElement<Self::Field>],
        _rap_challenges: &Self::RAPChallenges,
    ) -> Vec<FieldElement<Self::Field>> {
        let step = frame.get_evaluation_step(0);

        let prefix_flag = step.get_evaluation_element(0, 0);
        let next_prefix_flag = step.get_evaluation_element(1, 0);

        let two = Felt252::from(2);
        let one = Felt252::one();
        let bit_flag = prefix_flag - two * next_prefix_flag;

        let bit_constraint = bit_flag * (bit_flag - one);
        let zero_constraint = *step.get_evaluation_element(15, 0);

        vec![bit_constraint, zero_constraint]
    }

    fn boundary_constraints(
        &self,
        _rap_challenges: &Self::RAPChallenges,
    ) -> BoundaryConstraints<Self::Field> {
        // let a0 = BoundaryConstraint::new(1, 0, FieldElement::<Self::Field>::one());
        // let a1 = BoundaryConstraint::new(1, 1, FieldElement::<Self::Field>::one());

        BoundaryConstraints::from_constraints(vec![])
    }

    fn number_auxiliary_rap_columns(&self) -> usize {
        0
    }

    fn context(&self) -> &AirContext {
        &self.context
    }

    fn composition_poly_degree_bound(&self) -> usize {
        self.trace_length
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn pub_inputs(&self) -> &Self::PublicInputs {
        &()
    }
}

pub fn bit_prefix_flag_trace(num_steps: usize) -> TraceTable<Stark252PrimeField> {
    let step: Vec<Felt252> = vec![
        1031u64, 515, 257, 128, 64, 32, 16, 8, 4, 2, 1, 0, 0, 0, 0, 0,
    ]
    .iter()
    .map(|t| Felt252::from(*t))
    .collect();

    let data: Vec<Felt252> = iter::repeat(step).take(num_steps).flatten().collect();

    TraceTable::new(data, 1, 16)
}
