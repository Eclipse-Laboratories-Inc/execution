
#[cfg(test)]
mod tests {
    use tokio;

    use jmt_blake3::{
        JellyfishMerkleTree,
        mock::MockTreeStore,
        KeyHash,
        ValueHash,
    };

    use {
        solana_sdk::{
            account::Account,
            pubkey::Pubkey,
            clock::Slot,
        },
    };

    /// construct a empty tree
    #[tokio::test]
    async fn test_jmt_new_tree() {
        let db = MockTreeStore::default();
        let _tree = JellyfishMerkleTree::new(&db);
        assert_eq!(0, db.num_nodes().await);
    }

    /// insert blake3(account) and blake3(data)
    #[tokio::test]
    async fn test_jmt_build_account() {
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);

        let pub_key1 = Pubkey::new_unique();
        let pub_key2 = Pubkey::new_unique();

        let key1 = pubkey_to_h256(&pub_key1);
        let key2 = pubkey_to_h256(&pub_key2);
        let account1 = Account::new(1,2, &pub_key1);
        let account2 = Account::new(1,2, &pub_key2);
    }

    /// show proof of one account
    #[tokio::test]
    async fn test_jmt_show_proof() {}
}
