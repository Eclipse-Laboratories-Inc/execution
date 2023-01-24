use {
    solana_runtime::{
        accounts_update_notifier_interface::AccountsUpdateNotifierInterface,
        append_vec::{StoredAccountMeta, StoredMeta},
    },
    solana_sdk::{
        account::{AccountSharedData, ReadableAccount},
        clock::Slot,
        signature::Signature,
    },
};

#[derive(Debug)]
pub(crate) struct AccountsUpdateNotifierImpl {
}

impl AccountsUpdateNotifierImpl {
    pub fn new() -> Self {
        Self {}
    }
}
impl AccountsUpdateNotifierInterface for AccountsUpdateNotifierImpl {
    fn notify_account_update(
        &self,
        slot: Slot,
        meta: &StoredMeta,
        account: &AccountSharedData,
        txn_signature: &Option<&Signature>,
    ) {
       println!("notify_account_update: {}", slot);
    }
    fn notify_account_restore_from_snapshot(&self, slot: Slot, account: &StoredAccountMeta) {}

    fn notify_end_of_restore_from_snapshot(&self) {}
}

// pub type AccountsUpdateNotifier = Arc<RwLock<dyn AccountsUpdateNotifierInterface + Sync + Send>>;

