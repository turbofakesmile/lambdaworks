use crate::field::{
    element::FieldElement,
    traits::{IsTwoAdicField, RootsConfig},
};

use super::helpers::log2;

/// Calculates the (non-unitary) Discrete Fourier Transform of `input` via the DFT matrix.
pub fn naive_matrix_dft_test<F: IsTwoAdicField>(input: &[FieldElement<F>]) -> Vec<FieldElement<F>> {
    let n = input.len();
    let order = log2(n).unwrap();

    let twiddles = F::get_powers_of_primitive_root(order, n, RootsConfig::Natural).unwrap();

    let mut output = Vec::with_capacity(n);
    for row in 0..n {
        let mut sum = FieldElement::zero();

        for (col, element) in input.iter().enumerate() {
            let i = (row * col) % n; // w^i = w^(i mod n)
            sum += element.clone() * twiddles[i].clone();
        }

        output.push(sum);
    }

    output
}

#[cfg(test)]
mod fft_helpers_test {
    use crate::{
        field::test_fields::u64_test_field::{U64TestField, U64TestFieldElement},
        polynomial::Polynomial,
    };
    use proptest::prelude::*;

    use super::*;

    proptest! {
        // Property-based test that ensures dft() gives the same result as a naive polynomial evaluation.
        #[test]
        fn test_dft_same_as_eval(coeffs in U64TestField::vector_with_field_elements(8)) {
            let dft = naive_matrix_dft_test(&coeffs);

            let poly = Polynomial::new(&coeffs);
            let order = log2(coeffs.len()).unwrap();
            let twiddles = U64TestField::get_powers_of_primitive_root(order, coeffs.len(), RootsConfig::Natural).unwrap();
            let evals: Vec<U64TestFieldElement> = twiddles.iter().map(|x| poly.evaluate(x)).collect();

            prop_assert_eq!(evals, dft);
        }
    }
}
