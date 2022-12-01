use {
    clap::{value_t_or_exit, App, Arg, SubCommand},
    postgres::{Client, NoTls},
    serde_derive::{Deserialize, Serialize},
    shred_replay::{generate_hash, verify},
    solana_ledger::{blockstore::Blockstore, shred::Shred},
    std::{fs::File, io, io::Read, path::PathBuf},
    thiserror::Error,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayerPostgresConfig {
    /// The host name or IP of the PostgreSQL server
    pub host: Option<String>,

    /// The user name of the PostgreSQL server.
    pub user: Option<String>,

    pub password: Option<String>,
    pub dbname: Option<String>,
    /// The port number of the PostgreSQL database, the default is 5432
    pub port: Option<u16>,
}

#[derive(Error, Debug)]
pub enum ReplayerError {
    /// Error opening the configuration file; for example, when the file
    /// is not found or when the validator process has no permission to read it.
    #[error("Error opening config file. Error detail: ({0}).")]
    ConfigFileOpenError(#[from] io::Error),

    /// Error in reading the content of the config file or the content
    /// is not in the expected format.
    #[error("Error reading config file. Error message: ({msg})")]
    ConfigFileReadError { msg: String },
}

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

    let connection_str = format!(
        "host={} user={} password={} dbname={} port={}",
        config.host.as_ref().unwrap(),
        config.user.as_ref().unwrap(),
        config.password.as_ref().unwrap(),
        config.dbname.as_ref().unwrap(),
        config.port.as_ref().unwrap(),
    );

    let mut client = Client::connect(&connection_str, NoTls).unwrap();
    let stmt = "SELECT * FROM shred where slot = $1";
    let stmt = client.prepare(stmt).unwrap();

    let blockstore = Blockstore::open(ledger_path.as_path()).unwrap();
    let slot = blockstore.max_root() as i64;

    // Query shred by slot and update blockstore.
    for row in client.query(&stmt, &[&slot]).unwrap() {
        let payload: Vec<u8> = row.get(0);
        let shred = Shred::new_from_serialized_shred(payload).unwrap();
        let mut success = false;
        while !success {
            match blockstore.insert_shreds(vec![shred.clone()], None, false) {
                Ok(_) => success = true,
                Err(_) => success = false,
            }
        }
    }
 
    // Execute subcommand.
    match matches.subcommand() {
        ("", _) | ("replay", _) => {
            return;
        }
        ("verify", _) => {
            verify();
            return;
        }
        ("hash", _) => {
            generate_hash();
            return;
        }
        _ => unreachable!(),
    };
}
