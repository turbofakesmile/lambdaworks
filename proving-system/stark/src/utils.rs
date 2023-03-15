use super::{
    constraints::boundary::BoundaryConstraints, FE, FIELD_SUBGROUP_GENERATOR, MODULUS_MINUS_1,
    ORDER_OF_ROOTS_OF_UNITY_TRACE,
};
use lambdaworks_math::polynomial::Polynomial;
use std::ops::Div;

pub fn fibonacci_trace(initial_values: [FE; 2]) -> Vec<FE> {
    let mut ret: Vec<FE> = vec![];

    ret.push(initial_values[0].clone());
    ret.push(initial_values[1].clone());

    for i in 2..(ORDER_OF_ROOTS_OF_UNITY_TRACE as usize) {
        ret.push(ret[i - 1].clone() + ret[i - 2].clone());
    }

    ret
}

/// Returns the evaluation of the boundary constraint quotient polynomial on the provided ood evaluation point
/// required by DEEP FRI. This function is used by the verifier to check consistency between the trace
/// and the composition polynomial.
pub(crate) fn compute_boundary_quotient_ood_evaluation(
    constraints: &BoundaryConstraints<FE>,
    col: usize,
    primitive_root: &FE,
    trace_poly_ood_evaluation: &FE,
    ood_evaluation_point: &FE,
) -> FE {
    let domain = constraints.generate_roots_of_unity(primitive_root);
    let values = constraints.values(col);
    let zerofier = constraints.compute_zerofier(primitive_root);

    let poly = Polynomial::interpolate(&domain, &values);

    (trace_poly_ood_evaluation - poly.evaluate(ood_evaluation_point))
        / zerofier.evaluate(ood_evaluation_point)
}

/// Returns the evaluation of the transition quotient polynomial on the provided ood evaluation point
/// required by DEEP FRI. This function is used by the verifier to check consistency between the trace
/// and the composition polynomial.
pub fn compute_transition_quotient_ood_evaluation(
    primitive_root: &FE,
    trace_poly_ood_evaluations: &[FE],
    ood_evaluation_point: &FE,
) -> FE {
    let zerofier = compute_zerofier(primitive_root, ORDER_OF_ROOTS_OF_UNITY_TRACE as usize);

    (&trace_poly_ood_evaluations[2]
        - &trace_poly_ood_evaluations[1]
        - &trace_poly_ood_evaluations[0])
        / zerofier.evaluate(ood_evaluation_point)
}

pub(crate) fn compute_zerofier(primitive_root: &FE, root_order: usize) -> Polynomial<FE> {
    let roots_of_unity_vanishing_polynomial =
        Polynomial::new_monomial(FE::one(), root_order) - Polynomial::new(&[FE::one()]);
    let exceptions_to_vanishing_polynomial =
        Polynomial::new(&[-primitive_root.pow(root_order - 2), FE::one()])
            * Polynomial::new(&[-primitive_root.pow(root_order - 1), FE::one()]);

    roots_of_unity_vanishing_polynomial.div(exceptions_to_vanishing_polynomial)
}

// DEFINITION OF FUNCTIONS

pub fn generate_primitive_root(subgroup_size: u64) -> FE {
    let modulus_minus_1_field: FE = FE::new(MODULUS_MINUS_1);
    let subgroup_size: FE = subgroup_size.into();
    let generator_field: FE = FIELD_SUBGROUP_GENERATOR.into();
    let exp = (&modulus_minus_1_field) / &subgroup_size;
    generator_field.pow(exp.representative())
}

/// This functions takes a roots of unity and a coset factor
/// If coset_factor is 1, it's just expanding the roots of unity
/// w ^ 0, w ^ 1, w ^ 2 .... w ^ n-1
/// If coset_factor is h
/// h * w ^ 0, h * w ^ 1 .... h * w ^ n-1
pub fn generate_roots_of_unity_coset(coset_factor: u64, primitive_root: &FE) -> Vec<FE> {
    let coset_factor: FE = coset_factor.into();

    let mut numbers = vec![coset_factor.clone()];
    let mut exp: u64 = 1;
    let mut next_root = primitive_root.pow(exp) * &coset_factor;
    while next_root != coset_factor {
        numbers.push(next_root);
        exp += 1;
        next_root = primitive_root.pow(exp) * &coset_factor;
    }
    numbers
}
