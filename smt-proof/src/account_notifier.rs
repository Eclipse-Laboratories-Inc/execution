use std::fmt::{Debug, Formatter};
use {
    crate::shred_replay::ReplayerPostgresConfig,
    postgres::{Client, NoTls},
    solana_runtime::{
        accounts_update_notifier_interface::AccountsUpdateNotifierInterface,
        append_vec::{StoredAccountMeta, StoredMeta},
    },
    crossbeam_channel::{bounded, Receiver, RecvTimeoutError, Sender},
    solana_sdk::{
        account::{AccountSharedData, ReadableAccount},
        clock::Slot,
        signature::Signature,
    },
    std::{
        io,
        process::{exit},
        sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},},
        thread::{self, sleep, Builder, JoinHandle},
    },
    thiserror::Error,
};

/// GeyserPluginManager(ignore)
/// GeyserPlugin
///   SequencePostgresClient
///     SequencePostgresClientWorker
struct GeyserPlugin {
    // sequence_client: Option<SequencePostgresClient>,
    // exit_flag
    // client: Arc<RwLock<Client>>,
    // config: Option<ReplayerPostgresConfig>,
    // account convert method
    // notify 出现异常数据的处理
    // smt root
}


impl Debug for GeyserPlugin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GeyserPlugin for SMT replay")
    }
}

impl GeyserPlugin {
    fn new_with_config(_config: &ReplayerPostgresConfig) -> Self {
        // let config = config.as_ref().unwrap();
        // let connection_str = format!(
        //     "host={} user={} password={} dbname={} port={}",
        //     config.host.as_ref().unwrap(),
        //     config.user.as_ref().unwrap(),
        //     config.password.as_ref().unwrap(),
        //     config.dbname.as_ref().unwrap(),
        //     config.port.as_ref().unwrap(),
        // );
        // let client =
        //     Client::connect(&connection_str, NoTls).map_err(|_| ReplayRootError::DbConnectError {
        //         msg: format!("the config is {}", connection_str),
        //     }).unwrap();
        // Self {
        //     client: Arc::new(RwLock::new(client)),
        //     config: Some(config.clone()),
        // }
        Self {}
    }

}

#[derive(Error, Debug)]
pub enum ReplayRootError {
    #[error("Error connecting postgresdb Error message: ({msg})")]
    DbConnectError { msg: String },
}

#[derive(Debug)]
pub(crate) struct AccountsUpdateNotifierImpl {
    geyser_plugin: Arc<RwLock<GeyserPlugin>>,
}


impl AccountsUpdateNotifierImpl {
    pub fn new(config: &ReplayerPostgresConfig) -> Self {
        let geyser_plugin = GeyserPlugin::new_with_config(config);
        Self {
            geyser_plugin: Arc::new(RwLock::new(geyser_plugin))
        }
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

// /// SequenceClient, 1 recv, 1 worker thread
// #[allow(dead_code)]
// pub struct SequencePostgresClient {
//     worker: Option<JoinHandle<()>>,
//     exit_worker: Arc<AtomicBool>,
//     sender: Sender<DbWorkItem>,
//     slot: Arc<RwLock<i64>>,
//     smt_tree: Arc<RwLock<SMT>>,
// }
//
// impl SequencePostgresClient {
//     pub fn new(config: &GeyserPluginPostgresConfig) -> Self {
//         let (sender, receiver) = bounded(100);
//         let exit_worker = Arc::new(AtomicBool::new(false));
//         let exit_clone = exit_worker.clone();
//         let config = config.clone();
//         let slot: Arc<RwLock<i64>> = Arc::new(RwLock::new(1));
//         let slot_clone = slot.clone();
//         let smt_tree = Arc::new(RwLock::new(SMT::default()));
//         let smt_clone = smt_tree.clone();
//         let worker = Builder::new()
//             .name(format!("worker-sequence-account"))
//             .spawn(move || {
//                 let result = SequencePostgresClientWorker::new(config, slot_clone, smt_clone);
//
//                 match result {
//                     Ok(mut worker) => {
//                         worker.do_work(receiver, exit_clone, panic_on_db_errors);
//                     }
//                     Err(err) => {
//                         error!("Error when making connection to database: ({})", err);
//                         exit(1);
//                     }
//                 }
//             })
//             .unwrap();
//
//         Self {
//             worker: Some(worker),
//             exit_worker,
//             sender,
//             slot,
//             smt_tree,
//         }
//     }
//
//     pub fn join(&mut self) -> thread::Result<()> {
//         self.exit_worker.store(true, Ordering::Relaxed);
//         if let Some(handle) = self.worker.take() {
//             let result = handle.join();
//             if result.is_err() {
//                 error!("The worker thread has failed: {:?}", result);
//             }
//         }
//         Ok(())
//     }
//
//     pub fn update_account(
//         &mut self,
//         account: &ReplicaAccountInfoV2,
//         slot: u64,
//         is_startup: bool,
//     ) -> Result<(), GeyserPluginError> {
//         if !is_startup && account.txn_signature.is_none() {
//             // we are not interested in accountsdb internal bookeeping updates
//             return Ok(());
//         }
//
//         let wrk_item = DbWorkItem::UpdateAccount(Box::new(UpdateAccountRequest {
//             account: DbAccountInfo::new(account, slot),
//             is_startup,
//         }));
//
//         if let Err(err) = self.sender.send(wrk_item) {
//             return Err(GeyserPluginError::AccountsUpdateError {
//                 msg: format!(
//                     "Failed to update the account {:?}, error: {:?}",
//                     bs58::encode(account.pubkey()).into_string(),
//                     err
//                 ),
//             });
//         }
//
//         Ok(())
//     }
// }
//
// struct SequencePostgresClientWorker {
//     client: SimplePostgresClient,
//     slot: Arc<RwLock<i64>>,
//     smt_tree: Arc<RwLock<SMT>>,
// }
//
// impl SequencePostgresClientWorker {
//     fn new(
//         config: GeyserPluginPostgresConfig,
//         slot: Arc<RwLock<i64>>,
//         smt_tree: Arc<RwLock<SMT>>,
//     ) -> Result<Self, GeyserPluginError> {
//         let result = SimplePostgresClient::new(&config);
//         match result {
//             Ok(client) => Ok(SequencePostgresClientWorker { client, slot, smt_tree }),
//             Err(err) => {
//                 error!("Error in creating SequencePostgresClientWorker: {}", err);
//                 Err(err)
//             }
//         }
//     }
//
//     fn do_work(
//         &mut self,
//         receiver: Receiver<DbWorkItem>,
//         exit_worker: Arc<AtomicBool>,
//         panic_on_db_errors: bool,
//     ) -> Result<(), GeyserPluginError> {
//         while !exit_worker.load(Ordering::Relaxed) {
//             let work = receiver.recv_timeout(Duration::from_millis(500));
//             match work {
//                 Ok(work) => match work {
//                     DbWorkItem::UpdateAccount(request) => {
//                         let mut current_slot = self.slot.write().unwrap();
//                         // info!("do_work_recv {}, {}", *current_slot, request.account.slot);
//                         if *current_slot != request.account.slot {
//                             // info!("do_work_recv slot_changed {}, {}", *current_slot, request.account.slot);
//                             let smt_tree = self.smt_tree.read().unwrap();
//                             if let Err(err) = self.client.update_merkle_tree_root(*current_slot, smt_tree.root().as_slice()) {
//                                 info!("update_merkle_tree_root err");
//                             }
//                             *current_slot = request.account.slot;
//                         } else {
//                             let key_hash: H256 = request.account.key_hash();
//                             let val_hash: H256 = request.account.value_hash();
//                             if let Err(err) = self.smt_tree.write().unwrap().update(key_hash, val_hash) {
//                                 info!("update_merkle_tree_key_value err");
//                             }
//                         }
//                     }
//                     _ => (),
//                 },
//                 Err(err) => match err {
//                     RecvTimeoutError::Timeout => {
//                         continue;
//                     }
//                     _ => {
//                         error!("Error in receiving the item {:?}", err);
//                         if panic_on_db_errors {
//                             abort();
//                         }
//                         break;
//                     }
//                 },
//             }
//         }
//         Ok(())
//     }
// }
//
// trait HashAccount {
//     /// hash DbAccountInfo pubkey to sparse-merkle-tree key
//     /// Todo: pure synchronous jellyfish-merkle-tree data structure with blake3 hash method
//     fn key_hash(&self) -> H256;
//
//     /// hash DbAccountInfo(without slot property) to sparse-merkle-tree value
//     fn value_hash(&self) -> H256;
// }
//
// impl HashAccount for DbAccountInfo {
//     fn key_hash(&self) -> H256 {
//         // Hash
//         let res: Hash = blake3::hash(&self.pubkey);
//         // Hash to H256
//         H256::from(*res.as_bytes())
//     }
//     fn value_hash(&self) -> H256 {
//         let mut hasher = blake3::Hasher::new();
//         if self.lamports == 0 {
//             let res = hasher.finalize();
//             return H256::from(*res.as_bytes());
//         }
//
//         hasher.update(&self.lamports.to_le_bytes());
//         hasher.update(&self.rent_epoch.to_le_bytes());
//         hasher.update(&self.data);
//
//         if self.executable {
//             hasher.update(&[1u8; 1]);
//         } else {
//             hasher.update(&[0u8; 1]);
//         }
//         let res = hasher.finalize();
//         H256::from(*res.as_bytes())
//     }
// }
