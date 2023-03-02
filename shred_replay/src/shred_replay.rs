use assert_cmd::prelude::*;
use {
    postgres::{Client, NoTls},
    serde_derive::{Deserialize, Serialize},
    solana_ledger::{
        blockstore,
        blockstore::Blockstore,
        // genesis_utils::create_genesis_config,
        blockstore_options,
        shred::Shred,
    },
    solana_runtime::hardened_unpack::MAX_GENESIS_ARCHIVE_UNPACKED_SIZE,
    solana_sdk::genesis_config::GenesisConfig,
    std::{
        io,
        path::{Path, PathBuf},
        process::{Command, Output},
    },
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

    #[error("Error connecting postgresdb Error message: ({msg})")]
    DbConnectError { msg: String },

    #[error("Error initializing ledger Error message: ({msg})")]
    InitLedgerError { msg: String },

    #[error("Error initializing blockstore Error message: ({msg})")]
    InitBlockstoreError { msg: String },

    #[error("Error insert shreds into blockstore")]
    InsertShredError,
}

pub struct Replayer {
    client: Option<Client>,
    config: Option<ReplayerPostgresConfig>,
    ledger_path: Option<PathBuf>,
    genesis_path: Option<PathBuf>,
    blockstore: Option<Blockstore>,
}

impl Replayer {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
            ledger_path: None,
            genesis_path: None,
            blockstore: None,
        }
    }

    pub fn config(mut self, config: &ReplayerPostgresConfig) -> Self {
        self.config = Some(config.clone());
        self
    }

    pub fn ledger_path(mut self, ledger_path: &PathBuf) -> Self {
        self.ledger_path = Some(ledger_path.clone());
        self
    }

    pub fn genesis_path(mut self, genesis_path: &PathBuf) -> Self {
        self.genesis_path = Some(genesis_path.clone());
        self
    }

    pub fn connect_db(&mut self) -> Result<(), ReplayerError> {
        let config = self.config.as_ref().unwrap();
        let connection_str = format!(
            "host={} user={} password={} dbname={} port={}",
            config.host.as_ref().unwrap(),
            config.user.as_ref().unwrap(),
            config.password.as_ref().unwrap(),
            config.dbname.as_ref().unwrap(),
            config.port.as_ref().unwrap(),
        );
        let client =
            Client::connect(&connection_str, NoTls).map_err(|_| ReplayerError::DbConnectError {
                msg: format!("the config is {}", connection_str),
            })?;

        self.client = Some(client);
        Ok(())
    }

    pub fn init_ledger(&mut self) -> Result<(), ReplayerError> {
        if self.ledger_path.as_ref().unwrap().exists() {
        } else {
            // let genesis_config = create_genesis_config(100).genesis_config;
            // let origin_legder_path = Path::new("./test-ledger");
            let genesis_config = GenesisConfig::load(&self.genesis_path.as_ref().unwrap().as_path()).unwrap();
            let _last_hash = blockstore::create_new_ledger(
                self.ledger_path.as_ref().unwrap().as_path(),
                &genesis_config,
                MAX_GENESIS_ARCHIVE_UNPACKED_SIZE,
                blockstore_options::LedgerColumnOptions::default(),
            );
        };
        Ok(())
    }

    /// load shred from postgres by slot, order by index asc
    fn load_shred_from_pg(&mut self, slot: u64) -> Vec<Shred> {
        let mut shreds: Vec<Shred> = Vec::new();
        let stmt = "SELECT slot, entry_index, entry FROM entry where slot <= $1";
        let client = self.client.as_mut().unwrap();
        let stmt = client.prepare(stmt).unwrap();

        let result = client.query(&stmt, &[&(slot as i64)]);
        if result.is_err() {}

        for row in result.unwrap() {
            let payload: Vec<u8> = row.get(2);
            let result = Shred::new_from_serialized_shred(payload);
            if result.is_err() {}
            shreds.push(result.unwrap());
        }
        shreds
    }

    pub fn setup_blockstore(&mut self) -> Result<(), ReplayerError> {
        let blockstore = Blockstore::open(self.ledger_path.as_ref().unwrap()).map_err(|e| {
            ReplayerError::InitBlockstoreError {
                msg: { e.to_string() },
            }
        })?;
        self.blockstore = Some(blockstore);
        Ok(())
    }

    /// Query shred by slot and update blockstore.
    pub fn insert_shred_endwith_slot(&mut self, slot: u64) -> Result<(), ReplayerError> {
        let shreds = self.load_shred_from_pg(slot);
        self.blockstore
            .as_mut()
            .unwrap()
            .insert_shreds(shreds, None, false)
            .map_err(|_| ReplayerError::InsertShredError)?;
        Ok(())
    }
}

pub fn run_ledger_tool(args: &[&str]) -> Output {
    Command::cargo_bin("solana-ledger-tool")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}
