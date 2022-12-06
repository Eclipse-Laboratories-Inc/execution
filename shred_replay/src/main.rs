use log::error;
use {
    clap::{value_t_or_exit, App, Arg, SubCommand},
    std::{fs::File, io::Read, path::PathBuf},
    // shred_replay::{generate_hash, verify},
    shred_replay::shred_replay::{
        ReplayerPostgresConfig,
        ReplayerError,
        Replayer,
        run_ledger_tool
    },
};

fn main() {
    let matches = App::new("solana-replayer")
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
        .after_help("The default subcommand is replay")
        .subcommand(SubCommand::with_name("replay").about("replay shred and update ledger"))
        .subcommand(SubCommand::with_name("verify").about("Replay shred and verify the ledger"))
        .subcommand(SubCommand::with_name("hash").about("Replay shred and generate bank hash"))
        .get_matches();

    let config_file = value_t_or_exit!(matches, "config_file", PathBuf);
    let ledger_path = value_t_or_exit!(matches, "ledger_path", PathBuf);

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
        .ledger_path(&ledger_path);

    if let Err(e) = replayer.connect_db() {
        error!("{}", e);
    };
    if let Err(e) = replayer.init_ledger() {
        error!("{}", e);
    }
    if let Err(e) = replayer.setup_blockstore() {
        error!("{}", e);
    };
    let slot = 1u64;
    if let Err(e) = replayer.insert_shred_endwith_slot(slot) {
        error!("{}", e);
    }

    // Execute subcommand.
    match matches.subcommand() {
        ("verify", _) => {
            let src_slot_output = run_ledger_tool(&[
                "-l",
                &ledger_path.as_path().display().to_string(),
                "verify",
                ]);
            assert!(src_slot_output.status.success());
            println!("{}", String::from_utf8(src_slot_output.stdout).unwrap());
            return;
        }
        ("bank-hash", _) => {
            let src_slot_output = run_ledger_tool(&[
                "-l",
                &ledger_path.as_path().display().to_string(),
                "bank-hash"]);
            assert!(src_slot_output.status.success());
            println!("{}", String::from_utf8(src_slot_output.stdout).unwrap());
            return;
        }
        _ => unreachable!(),
    };
}
