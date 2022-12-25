#[cfg(test)]
mod tests {
    use {
        sparse_merkle_tree::{
            SparseMerkleTree,
            blake2b::Blake2bHasher,
            default_store::DefaultStore,
            H256,
        },
        hex::decode,
    };

    use {
        solana_sdk::{
            account::Account,
            pubkey::Pubkey,
            clock::Slot,
        },
        solana_program::hash::Hash,
        solana_runtime::accounts_db::AccountsDb,
    };

    type SMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

    fn str_to_h256(src: &str) -> H256 {
        let src = decode(src).unwrap();
        assert_eq!(src.len(), 32);
        let data: [u8; 32] = src.try_into().unwrap();
        H256::from(data)
    }

    #[test]
    fn test_ckb_smt_verify_1() {
        let mut tree = SMT::default();
        let key = str_to_h256("381dc5391dab099da5e28acd1ad859a051cf18ace804d037f12819c6fbc0e18b");
        let val = str_to_h256("9158ce9b0e11dd150ba2ae5d55c1db04b1c5986ec626f2e38a93fe8ad0b2923b");
        tree.update(key, val).unwrap();
        assert!(!tree.root().is_zero());
    }

    // convert key type to H256
    fn pubkey_to_h256(key: &Pubkey) -> H256 {
        H256::from(key.to_bytes())
    }

    // convert value type to H256
    fn hash_to_h256(hash: &Hash) -> H256 {
        H256::from(hash.to_bytes())
    }

    #[test]
    fn test_smt_build_sol_account() {

        let mut tree = SMT::default();

        // construct AccountSharedData
        // key is Pubkey
        // val is hash(account)

        let pub_key1 = Pubkey::new_unique();
        let pub_key2 = Pubkey::new_unique();

        let key1 = pubkey_to_h256(&pub_key1);
        let key2 = pubkey_to_h256(&pub_key2);
        let account1 = Account::new(1,2, &pub_key1);
        let account2 = Account::new(1,2, &pub_key2);

        let slot : Slot = 0;
        let hash1 = AccountsDb::hash_account(slot, &account1, &pub_key1);
        let hash2 = AccountsDb::hash_account(slot, &account1, &pub_key1);
        let val1 = hash_to_h256(&hash1);
        let val2 = hash_to_h256(&hash2);

        tree.update(key1, val1).unwrap();
        tree.update(key2, val2).unwrap();
        assert!(!tree.root().is_zero());
    }
}
