use alloc::vec::Vec;
use lambdaworks_math::field::{element::FieldElement as FE, traits::IsPrimeField};

/// Parameters for Poseidon
/// MDS constants and rounds constants are stored as references to slices
/// representing matrices of `N_MDS_MATRIX_ROWS * N_MDS_MATRIX_COLS` and
/// `N_ROUND_CONSTANTS_ROWS * N_ROUND_CONSTANTS_COLS` respectively.
/// We use this representation rather than an array because we can't use the
/// associated constants for dimension, requiring many generic parameters
/// otherwise.
pub trait PermutationParameters {
    type F: IsPrimeField + 'static;

    const RATE: usize;
    const CAPACITY: usize;
    const ALPHA: u32;
    const N_FULL_ROUNDS: usize;
    const N_PARTIAL_ROUNDS: usize;
    const STATE_SIZE: usize = Self::RATE + Self::CAPACITY;

    const MDS_MATRIX: &'static [FE<Self::F>];
    const N_MDS_MATRIX_ROWS: usize;
    const N_MDS_MATRIX_COLS: usize;

    /// Add Round Constants optimized using  Appendix B specification of  Poseidon paper (available at https://eprint.iacr.org/2019/458.pdf). 
    /// If you have the non optimized ones, edit the file in ```poseidon/optimized_constant_generator.rs```, run ```cargo run --bin poseidon-constant-generator``` in the crypto package. This will generate a file opt_constants.txt which you can copy and paste in your PermutationParameters
    const ROUND_CONSTANTS: &'static [FE<Self::F>];
    const N_ROUND_CONSTANTS_ROWS: usize;
    const N_ROUND_CONSTANTS_COLS: usize;

    /// This is the mix function that operates with the MDS matrix
    /// Round Constants are sometimes picked to simplify this function,
    /// so it can be redefined by each set of permutation parameters if a simplification can be made to make it faster. Notice in that case, MDS constants may not be used.
    fn mix(state: &mut [FE<Self::F>]) {
        let mut new_state: Vec<FE<Self::F>> = Vec::with_capacity(Self::STATE_SIZE);
        for i in 0..Self::STATE_SIZE {
            let mut new_e = FE::zero();
            for (j, current_state) in state.iter().enumerate() {
                let mut mij = Self::MDS_MATRIX[i * Self::N_MDS_MATRIX_COLS + j].clone();
                mij *= current_state;
                new_e += mij;
            }
            new_state.push(new_e);
        }
        state.clone_from_slice(&new_state[0..Self::STATE_SIZE]);
    }
}
