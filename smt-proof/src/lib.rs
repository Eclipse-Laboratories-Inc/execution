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
        blake3::{self, Hash},
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
        assert_eq!(tree.store().branches_map().len(), 0);
        assert_eq!(tree.store().leaves_map().len(), 0);
        assert_eq!(tree.root(), &H256::zero());
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

    /// convert solana Pubkey to H256
    fn pubkey_to_h256(key: &Pubkey) -> H256 {
        H256::from(key.to_bytes())
    }

    /// convert solana hash to H256
    fn blake3hash_to_h256(hash: &Hash) -> H256 {
        H256::from(*hash.as_bytes())
    }

    #[test]
    fn test_smt_build_verify() {
        let mut tree = SMT::default();

        let pub_key1 = Pubkey::new_unique();
        let pub_key2 = Pubkey::new_unique();

        let key1 = pubkey_to_h256(&pub_key1);
        let key2 = pubkey_to_h256(&pub_key2);
        let b1_raw = b"binary1";
        let b2_raw = b"binary2";
        let bl3_hash_1 = blake3::hash(b1_raw.as_slice());
        let bl3_hash_2 = blake3::hash(b2_raw.as_slice());
        let val1 = blake3hash_to_h256(&bl3_hash_1);
        let val2 = blake3hash_to_h256(&bl3_hash_2);
        tree.update(key1, val1).unwrap();
        tree.update(key2, val2).unwrap();

        let proof = tree.merkle_proof(vec![key1, key2]).expect("proof");
        assert!(proof
            .verify::<Blake2bHasher>(tree.root(), vec![(key1, val1), (key2, val2)])
            .expect("verify"));
    }
}
