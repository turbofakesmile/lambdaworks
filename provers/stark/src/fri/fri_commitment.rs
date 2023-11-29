use lambdaworks_crypto::merkle_tree::{merkle::MerkleTree, traits::IsMerkleTreeBackend};
use lambdaworks_math::{
    field::{
        element::FieldElement,
        traits::{IsFFTField, IsField, IsSubFieldOf},
    },
    traits::Serializable,
};

#[derive(Clone)]
pub struct FriLayer<F, E, B>
where
    F: IsField + IsSubFieldOf<E>,
    E: IsField,
    FieldElement<F>: Serializable,
    B: IsMerkleTreeBackend,
{
    pub evaluation: Vec<FieldElement<E>>,
    pub merkle_tree: MerkleTree<B>,
    pub coset_offset: FieldElement<F>,
    pub domain_size: usize,
}

impl<F, E, B> FriLayer<F, E, B>
where
    F: IsField + IsSubFieldOf<E>,
    E: IsField,
    FieldElement<F>: Serializable,
    B: IsMerkleTreeBackend,
{
    pub fn new(
        evaluation: &[FieldElement<E>],
        merkle_tree: MerkleTree<B>,
        coset_offset: FieldElement<F>,
        domain_size: usize,
    ) -> Self {
        Self {
            evaluation: evaluation.to_vec(),
            merkle_tree,
            coset_offset,
            domain_size,
        }
    }
}
