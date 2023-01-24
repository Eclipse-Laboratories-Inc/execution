//
use {
    crate::ledger_path::parse_ledger_path,
    clap::{
        value_t, value_t_or_exit, values_t_or_exit, App,
        Arg, ArgMatches, SubCommand
    },
    solana_sdk::{
        clock::Slot,
        genesis_config::{GenesisConfig},
    },
    log::{info, error},
    solana_ledger::{
        bank_forks_utils,
        blockstore::{Blockstore},
        blockstore_options::{
            AccessType, BlockstoreOptions, BlockstoreRecoveryMode, LedgerColumnOptions,
            ShredStorageType,
        },
        blockstore_processor::{self, BlockstoreProcessorError, ProcessOptions},
    },
    solana_runtime::{
        accounts_background_service::{
            AbsRequestHandler, AbsRequestSender, AccountsBackgroundService,
        },
        accounts_db::{AccountsDbConfig, FillerAccountsConfig},
        accounts_index::{AccountsIndexConfig},
        bank_forks::BankForks,
        hardened_unpack::{open_genesis_config, MAX_GENESIS_ARCHIVE_UNPACKED_SIZE},
        runtime_config::RuntimeConfig,
    },
    solana_clap_utils::{
        // input_parsers::{cluster_type_of, pubkey_of, pubkeys_of},
        input_validators::{
            is_parsable // , is_pow2, is_pubkey, is_pubkey_or_keypair, is_slot, is_valid_percentage,
        },
    },
    std::{
        path::{Path},
        process::{exit},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, RwLock,
        },
        fs::File, io::Read
    }
};

mod ledger_path;
mod shred_replay;

const DEFAULT_LEDGER_TOOL_ROCKS_FIFO_SHRED_STORAGE_SIZE_BYTES: u64 = u64::MAX;

fn main() {

    let starting_slot_arg = Arg::with_name("starting_slot")
            .long("starting_slot")
            .value_name("SLOT")
            .takes_value(true)
            .required(true)
            .default_value("0")
            .help("Start at this slot");
    let ending_slot_arg = Arg::with_name("ending_slot")
        .long("ending_slot")
        .value_name("SLOT")
        .takes_value(true)
        .required(true)
        .help("The last slot to iterate to");

    let accounts_filler_count = Arg::with_name("accounts_filler_count")
        .long("accounts-filler-count")
        .value_name("COUNT")
        .validator(is_parsable::<usize>)
        .takes_value(true)
        .default_value("0")
        .help("How many accounts to add to stress the system. Accounts are ignored in operations related to correctness.");
    let accounts_filler_size = Arg::with_name("accounts_filler_size")
        .long("accounts-filler-size")
        .value_name("BYTES")
        .validator(is_parsable::<usize>)
        .takes_value(true)
        .default_value("0")
        .requires("accounts_filler_count")
        .help("Size per filler account in bytes.");

    let default_genesis_archive_unpacked_size = MAX_GENESIS_ARCHIVE_UNPACKED_SIZE.to_string();
    let max_genesis_archive_unpacked_size_arg = Arg::with_name("max_genesis_archive_unpacked_size")
        .long("max-genesis-archive-unpacked-size")
        .value_name("NUMBER")
        .takes_value(true)
        .default_value(&default_genesis_archive_unpacked_size)
        .help("maximum total uncompressed size of unpacked genesis archive");

    let matches = App::new("solana-smt-replayer")
        .about("Replayer")
        .version("0.1")
        .arg(
            Arg::with_name("config_file")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .takes_value(true)
                .required(true)
                .default_value("config.json")
                .help("Configuration file to use"),
        )
        .arg(
            Arg::with_name("ledger_path")
                .short("l")
                .long("ledger")
                .value_name("DIR")
                .takes_value(true)
                .required(true)
                .default_value("ledger")
                .help("Use DIR as ledger location"),
        )
        .arg(
            Arg::with_name("out_ledger_path")
                .short("o")
                .long("out_ledger")
                .value_name("DIR")
                .takes_value(true)
                .required(true)
                .default_value("/tmp/out-ledger")
                .help("Use DIR as ledger location"),
        )
        .arg(
            Arg::with_name("wal_recovery_mode")
                .long("wal-recovery-mode")
                .value_name("MODE")
                .takes_value(true)
                .global(true)
                .possible_values(&[
                    "tolerate_corrupted_tail_records",
                    "absolute_consistency",
                    "point_in_time",
                    "skip_any_corrupted_record"])
                .help(
                    "Mode to recovery the ledger db write ahead log"
                ),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .global(true)
                .multiple(true)
                .takes_value(false)
                .help("Show additional information where supported"),
        )
        .subcommand(
            SubCommand::with_name("verify")
                .about("Verify the ledger")
                .arg(&starting_slot_arg)
                .arg(&ending_slot_arg)
                .arg(&accounts_filler_count)
                .arg(&accounts_filler_size)
                .arg(&max_genesis_archive_unpacked_size_arg)
        )
        .after_help("The default subcommand is replay")
        .get_matches();

    let config_file = parse_ledger_path(&matches, "config_file");
    let ledger_path = parse_ledger_path(&matches, "ledger_path");
    let out_ledger_path = parse_ledger_path(&matches, "out_ledger_path");
    println!("{:?}", config_file);
    println!("{:?}", out_ledger_path);

    let mut file = File::open(config_file.as_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let config: shred_replay::ReplayerPostgresConfig = serde_json::from_str(&contents)
        .map_err(|err| shred_replay::ReplayerError::ConfigFileReadError {
            msg: format!(
                "The config file is not in the JSON format expected: {:?}",
                err
            ),
        })
        .unwrap();

    let wal_recovery_mode = matches
        .value_of("wal_recovery_mode")
        .map(BlockstoreRecoveryMode::from);
    // let verbose_level = matches.occurrences_of("verbose");

    let shred_storage_type = match ShredStorageType::from_ledger_path(
        &ledger_path,
        DEFAULT_LEDGER_TOOL_ROCKS_FIFO_SHRED_STORAGE_SIZE_BYTES,
    ) {
        Some(s) => s,
        None => {
            error!("Shred storage type cannot be inferred, the default RocksLevel will be used");
            ShredStorageType::RocksLevel
        }
    };


    match matches.subcommand() {
        ("verify", Some(arg_matches)) => {

            let mut replayer = shred_replay::Replayer::new().config(&config).ledger_path(&out_ledger_path);

            if let Err(e) = replayer.connect_db() {
                eprintln!("Failed to connect pg {}", e);
                exit(1);
            };

            let mut accounts_index_config = AccountsIndexConfig::default();

            let starting_slot = value_t_or_exit!(arg_matches, "starting_slot", Slot);
            let ending_slot = value_t_or_exit!(arg_matches, "ending_slot", Slot);
            println!("{} {}", starting_slot, ending_slot);

            if let Some(bins) = value_t!(arg_matches, "accounts_index_bins", usize).ok() {
                accounts_index_config.bins = Some(bins);
            }

            let filler_accounts_config = FillerAccountsConfig {
                count: value_t_or_exit!(arg_matches, "accounts_filler_count", usize),
                size: value_t_or_exit!(arg_matches, "accounts_filler_size", usize),
            };

            let accounts_db_config = Some(AccountsDbConfig {
                index: Some(accounts_index_config),
                accounts_hash_cache_path: Some(ledger_path.clone()),
                filler_accounts_config,
                skip_rewrites: arg_matches.is_present("accounts_db_skip_rewrites"),
                ancient_append_vecs: arg_matches.is_present("accounts_db_ancient_append_vecs"),
                skip_initial_hash_calc: arg_matches
                    .is_present("accounts_db_skip_initial_hash_calculation"),
                ..AccountsDbConfig::default()
            });

            let process_options = ProcessOptions {
                new_hard_forks: hardforks_of(arg_matches, "hard_forks"),
                poh_verify: !arg_matches.is_present("skip_poh_verify"),
                on_halt_store_hash_raw_data_for_debug: arg_matches
                    .is_present("halt_at_slot_store_hash_raw_data"),
                // ledger tool verify always runs the accounts hash calc at the end of processing the blockstore
                run_final_accounts_hash_calc: true,
                halt_at_slot: value_t!(arg_matches, "halt_at_slot", u64).ok(),
                debug_keys: None,
                accounts_db_caching_enabled: !arg_matches.is_present("no_accounts_db_caching"),
                limit_load_slot_count_from_snapshot: value_t!(
                        arg_matches,
                        "limit_load_slot_count_from_snapshot",
                        usize
                    )
                    .ok(),
                accounts_db_config,
                verify_index: arg_matches.is_present("verify_accounts_index"),
                allow_dead_slots: arg_matches.is_present("allow_dead_slots"),
                accounts_db_test_hash_calculation: arg_matches
                    .is_present("accounts_db_test_hash_calculation"),
                accounts_db_skip_shrink: arg_matches.is_present("accounts_db_skip_shrink"),
                runtime_config: RuntimeConfig {
                    bpf_jit: !arg_matches.is_present("no_bpf_jit"),
                    ..RuntimeConfig::default()
                },
                ..ProcessOptions::default()
            };

            let genesis_config = open_genesis_config_by(&ledger_path, arg_matches);
            if let Err(e) = replayer.init_ledger(&genesis_config) {
                eprintln!("Failed to init new ledger{}", e);
                exit(1);
            };

            let mut blockstore = open_blockstore(
                &out_ledger_path,
                AccessType::Primary,
                wal_recovery_mode,
                &shred_storage_type,
            );

            if let Err(e) = replayer.insert_shred_endwith_slot(ending_slot, &mut blockstore) {
                eprintln!("Failed to insert shred in pg: {}", e);
                exit(1);
            };

            let result = load_bank_forks(
                &genesis_config,
                &blockstore,
                process_options,
            );

            println!("{}", result.is_ok());
        }
        _ => {

        }
    }
}

fn open_blockstore(
    ledger_path: &Path,
    access_type: AccessType,
    wal_recovery_mode: Option<BlockstoreRecoveryMode>,
    shred_storage_type: &ShredStorageType,
) -> Blockstore {
    match Blockstore::open_with_options(
        ledger_path,
        BlockstoreOptions {
            access_type,
            recovery_mode: wal_recovery_mode,
            enforce_ulimit_nofile: true,
            column_options: LedgerColumnOptions {
                shred_storage_type: shred_storage_type.clone(),
                ..LedgerColumnOptions::default()
            },
        },
    ) {
        Ok(blockstore) => blockstore,
        Err(err) => {
            eprintln!("Failed to open ledger at {:?}: {:?}", ledger_path, err);
            exit(1);
        }
    }
}

// This function is duplicated in validator/src/main.rs...
fn hardforks_of(matches: &ArgMatches<'_>, name: &str) -> Option<Vec<Slot>> {
    if matches.is_present(name) {
        Some(values_t_or_exit!(matches, name, Slot))
    } else {
        None
    }
}

fn load_bank_forks(
    genesis_config: &GenesisConfig,
    blockstore: &Blockstore,
    process_options: ProcessOptions,
) -> Result<Arc<RwLock<BankForks>>, BlockstoreProcessorError> {

    let starting_slot = 0; // default start check with genesis

    if let Some(halt_slot) = process_options.halt_at_slot {
        // Check if we have the slot data necessary to replay from starting_slot to >= halt_slot.
        //  - This will not catch the case when loading from genesis without a full slot 0.
        if !blockstore.slot_range_connected(starting_slot, halt_slot) {
            eprintln!(
                "Unable to load bank forks at slot {} due to disconnected blocks.",
                halt_slot,
            );
            exit(1);
        }
    }

    let account_paths =
    {
        let non_primary_accounts_path = blockstore.ledger_path().join("accounts.ledger-tool");
        info!(
            "Default accounts path is switched aligning with Blockstore's secondary access: {:?}",
            non_primary_accounts_path
        );

        if non_primary_accounts_path.exists() {
            info!("Clearing {:?}", non_primary_accounts_path);
            if let Err(err) = std::fs::remove_dir_all(&non_primary_accounts_path) {
                eprintln!(
                    "error deleting accounts path {:?}: {}",
                    non_primary_accounts_path, err
                );
                exit(1);
            }
        }

        vec![non_primary_accounts_path]
    };

    let snapshot_config = None;
    let (bank_forks, leader_schedule_cache, ..) =
        bank_forks_utils::load_bank_forks(
            genesis_config,
            blockstore,
            account_paths,
            None,
            snapshot_config.as_ref(),
            &process_options,
            None,
            None,
        );

    let pruned_banks_receiver =
        AccountsBackgroundService::setup_bank_drop_callback(bank_forks.clone());
    let abs_request_handler = AbsRequestHandler {
        snapshot_request_handler: None,
        pruned_banks_receiver,
    };
    let exit = Arc::new(AtomicBool::new(false));
    let accounts_background_service = AccountsBackgroundService::new(
        bank_forks.clone(),
        &exit,
        abs_request_handler,
        process_options.accounts_db_caching_enabled,
        process_options.accounts_db_test_hash_calculation,
        None,
    );

    blockstore_processor::process_blockstore_from_root(
        blockstore,
        &bank_forks,
        &leader_schedule_cache,
        &process_options,
        None,
        None,
        &AbsRequestSender::default(),
    )?;

    exit.store(true, Ordering::Relaxed);
    accounts_background_service.join().unwrap();

    Ok(bank_forks)
    // result
}

fn open_genesis_config_by(ledger_path: &Path, matches: &ArgMatches<'_>) -> GenesisConfig {
    let max_genesis_archive_unpacked_size =
        value_t_or_exit!(matches, "max_genesis_archive_unpacked_size", u64);
    open_genesis_config(ledger_path, max_genesis_archive_unpacked_size)
}