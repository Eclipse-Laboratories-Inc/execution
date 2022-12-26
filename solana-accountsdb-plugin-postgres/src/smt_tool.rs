use {
    sparse_merkle_tree::{
        SparseMerkleTree,
        blake2b::Blake2bHasher,
        default_store::DefaultStore,
        H256,
    },
};

use {
    solana_sdk::{
        pubkey::Pubkey,
        clock::Slot,
    },
    solana_program::hash::Hash,
};

pub type SMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

// convert key type to H256
pub fn pubkey_to_h256(slice: &Vec<u8>) -> H256 {
    let fixed_array: [u8; 32] = slice.into();
    H256::from(fixed_array);
}

// convert value type to H256
pub fn hash_to_h256(hash: &Hash) -> H256 {
    H256::from(hash.to_bytes())
}
