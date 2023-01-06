/// SparseMerkleTree api
/// 1. new
/// 2. updates
/// 3. get_proof_with
/// 4. provide intermediate for conversion
///
/// type conversion notes
/// Provide an intermediate type for conversion, instead of directly using solana's type
/// to avoid circular reference.

#[cfg(test)]
mod tests {
    use {
        hex::decode,
        solana_sdk::pubkey::Pubkey,
        sparse_merkle_tree::{
            blake2b::Blake2bHasher, default_store::DefaultStore, SparseMerkleTree, H256,
        },
    };
    type SMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

    /// convert basic string to H256 type
    fn str_to_h256(src: &str) -> H256 {
        let src = decode(src).unwrap();
        assert_eq!(src.len(), 32);
        let data: [u8; 32] = src.try_into().unwrap();
        H256::from(data)
    }

    #[test]
    fn test_smt_new_empty_tree() {
        let tree = SMT::default();
        assert!(tree.is_empty());
    }

    #[test]
    fn test_ckb_smt_update_1_item() {
        let mut tree = SMT::default();
        let key = str_to_h256("381dc5391dab099da5e28acd1ad859a051cf18ace804d037f12819c6fbc0e18b");
        let val = str_to_h256("9158ce9b0e11dd150ba2ae5d55c1db04b1c5986ec626f2e38a93fe8ad0b2923b");
        let res1 = tree.update(key, val);
        assert!(res1.is_ok());

        let res2 = tree.get(&key);
        assert!(res2.is_ok());
        assert_eq!(val, res2.unwrap());
    }
}
