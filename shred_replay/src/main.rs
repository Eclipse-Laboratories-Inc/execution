use log::error;
use {
    clap::{value_t_or_exit, App, Arg, SubCommand},
    // shred_replay::{generate_hash, verify},
    shred_replay::shred_replay::{
        run_ledger_tool, Replayer, ReplayerError, ReplayerPostgresConfig,
    },
    std::{fs::File, io::Read, path::PathBuf},
};

fn main() {
    let matches = App::new("solana-shred-replayer")
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
            Arg::with_name("genesis_path")
                .short("g")
                .long("genesis")
                .value_name("GENESIS_PATH")
                .takes_value(true)
                .required(true)
                .default_value("ledger")
                .help("Use GENESIS_PATH as genesis path"),
        )
        .after_help("The default subcommand is replay")
        .subcommand(SubCommand::with_name("replay").about("replay shred and update ledger"))
        .subcommand(SubCommand::with_name("verify").about("Replay shred and verify the ledger"))
        .subcommand(SubCommand::with_name("hash").about("Replay shred and generate bank hash"))
        .get_matches();

    let config_file = value_t_or_exit!(matches, "config_file", PathBuf);
    let ledger_path = value_t_or_exit!(matches, "ledger_path", PathBuf);
    let genesis_path = value_t_or_exit!(matches, "genesis_path", PathBuf);
    // let config_file = PathBuf::from("./solana-accountsdb-plugin-postgres/scripts/geyser.json");
    // let ledger_path = PathBuf::from("replay-ledger");
    // let genesis_path = PathBuf::from("test-ledger");

    let mut file = File::open(config_file.as_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let config: ReplayerPostgresConfig = serde_json::from_str(&contents)
        .map_err(|err| ReplayerError::ConfigFileReadError {
            msg: format!(
                "The config file is not in the JSON format expected: {:?}",
                err
            ),
        })
        .unwrap();

    let mut replayer = Replayer::new()
        .config(&config)
        .ledger_path(&ledger_path)
        .genesis_path(&genesis_path);

    if let Err(e) = replayer.connect_db() {
        error!("{}", e);
    };
    if let Err(e) = replayer.init_ledger() {
        error!("{}", e);
    }
    if let Err(e) = replayer.setup_blockstore() {
        error!("{}", e);
    };
    // We query last verified slot from DB.
    let slot = replayer.query_last_verified_slot();
    let start_slot = match slot {
        Some(slot) => slot + 1,
        None => 1,
    };

    println!("Start to verify shred from slot: {}", start_slot);
    replayer.insert_shred_startwith_slot(start_slot);

    // // Execute subcommand.
    // match matches.subcommand() {
    //     ("verify", _) => {
    //         let src_slot_output =
    //             run_ledger_tool(&["-l", &ledger_path.as_path().display().to_string(), "verify"]);
    //         assert!(src_slot_output.status.success());
    //         println!("{}", String::from_utf8(src_slot_output.stdout).unwrap());
    //         return;
    //     }
    //     ("bank-hash", _) => {
    //         let src_slot_output = run_ledger_tool(&[
    //             "-l",
    //             &ledger_path.as_path().display().to_string(),
    //             "bank-hash",
    //         ]);
    //         assert!(src_slot_output.status.success());
    //         println!("{}", String::from_utf8(src_slot_output.stdout).unwrap());
    //         return;
    //     }
    //     _ => unreachable!(),
    // };
}
