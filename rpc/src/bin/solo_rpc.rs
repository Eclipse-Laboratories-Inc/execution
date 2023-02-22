use {
    solana_rpc::{
        rpc::{create_validator_exit, JsonRpcConfig},
        optimistically_confirmed_bank_tracker::OptimisticallyConfirmedBank,
        rpc_service::JsonRpcService,
        max_slots::MaxSlots,
    },
    solana_ledger::{
        blockstore::Blockstore,
        blockstore_processor::{BlockstoreProcessorError, ProcessOptions},
        bank_forks_utils,
        leader_schedule_cache::LeaderScheduleCache,
        genesis_utils::{create_genesis_config, GenesisConfigInfo},
    },
    solana_runtime::hardened_unpack::{open_genesis_config, MAX_GENESIS_ARCHIVE_UNPACKED_SIZE},
    solana_client::rpc_config::RpcContextConfig,
    solana_client::{connection_cache::ConnectionCache, rpc_cache::LargestAccountsCache},
    solana_gossip::{
        cluster_info::ClusterInfo,
        contact_info::ContactInfo,
        crds::GossipRoute,
        crds_value::{CrdsData, CrdsValue, SnapshotHashes},
    },
    solana_runtime::{bank::Bank, bank_forks::BankForks, commitment::BlockCommitmentCache,},
    
    solana_sdk::{
        genesis_config::{ClusterType, DEFAULT_GENESIS_ARCHIVE},
        signature::Signer,
        signer::keypair::Keypair,
        hash::Hash,
    },
    solana_streamer::socket::SocketAddrSpace,
    solana_send_transaction_service::send_transaction_service::{self, SendTransactionService},
    std::{
        collections::HashSet,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicBool, AtomicU64, Ordering},
            Arc, RwLock,
        },
        thread::{self, Builder, JoinHandle},
        net::{IpAddr, Ipv4Addr, SocketAddr},
    },
    solana_net_utils,
};

// 1. slot, height is 0
// 2. "Method not found" for some RPC, e.g. getSupply, getSlotLeader.
fn main() {
    let ledger_path = Path::new("./test-ledger");
    let genesis_config = open_genesis_config(&ledger_path, MAX_GENESIS_ARCHIVE_UNPACKED_SIZE);
    let blockstore = Arc::new(Blockstore::open(&ledger_path).unwrap());

    let account_paths = vec![PathBuf::new()];
    let process_options = ProcessOptions::default();
    let (bank_forks, _leader_schedule_cache, ..) =
        bank_forks_utils::load_bank_forks(
            &genesis_config,
            &blockstore,
            account_paths,
            None,
            None,
            &process_options,
            None,
            None,
        );

    let exit = Arc::new(AtomicBool::new(false));
    let validator_exit = create_validator_exit(&exit);
    let cluster_info = Arc::new(ClusterInfo::new(
        ContactInfo::default(),
        Arc::new(Keypair::new()),
        SocketAddrSpace::Unspecified,
    ));
    let ip_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let rpc_addr = SocketAddr::new(
        ip_addr,
        8899,
    );
    
    let block_commitment_cache = Arc::new(RwLock::new(BlockCommitmentCache::default()));
    let optimistically_confirmed_bank =
        OptimisticallyConfirmedBank::locked_from_bank_forks_root(&bank_forks);
    let connection_cache = Arc::new(ConnectionCache::default());
    let rpc_service = JsonRpcService::new(
        rpc_addr,
        JsonRpcConfig::default(),
        None,
        bank_forks,
        block_commitment_cache,
        blockstore,
        cluster_info,
        None,
        Hash::default(),
        &PathBuf::from("farf"),
        validator_exit,
        None,
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(true)),
        optimistically_confirmed_bank,
        send_transaction_service::Config {
            retry_rate_ms: 1000,
            leader_forward_count: 1,
            ..send_transaction_service::Config::default()
        },
        Arc::new(MaxSlots::default()),
        Arc::new(LeaderScheduleCache::default()),
        connection_cache,
        Arc::new(AtomicU64::default()),
    );

    println!("rpc: {}", rpc_addr);

    rpc_service.join().unwrap();
}