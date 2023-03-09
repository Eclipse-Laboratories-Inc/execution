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
    const VERIFY_INTERVAL_SLOTS: i64 = 50;
    const CREATE_SNAPSHOT_INTERVAL_SLOTS: i64 = 40;

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
            let genesis_config =
                GenesisConfig::load(&self.genesis_path.as_ref().unwrap().as_path()).unwrap();
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
        let stmt =
            "SELECT slot, entry_index, entry FROM entry where slot = $1 ORDER BY entry_index ASC";
        let client = self.client.as_mut().unwrap();
        let stmt = client.prepare(stmt).unwrap();

        // let mut cur_slot = 1;

        let result = client.query(&stmt, &[&(slot as i64)]);
        if result.is_err() {
            println!("query error for slot: {}", slot);
        }

        for row in result.unwrap() {
            let s: i64 = row.get(0);
            let ei: i64 = row.get(1);
            println!("slot: {}, entry_index: {}", s, ei);
            let payload: Vec<u8> = row.get(2);
            let result = Shred::new_from_serialized_shred(payload);
            if result.is_err() {
                println!("serialize shred error for slot: {}", slot);
            }
            shreds.push(result.unwrap());
        }
        shreds
    }

    pub fn query_last_verified_slot(&mut self) -> Option<u64> {
        let stmt = "SELECT slot, entry_index FROM replay ORDER BY slot DESC LIMIT 1";
        let client = self.client.as_mut().unwrap();
        let stmt = client.prepare(stmt).unwrap();
        let result = client.query(&stmt, &[]);
        if result.is_err() {
            println!("query replay record failed: {:?}", result.err());
            return None;
        }

        if result.as_ref().unwrap().is_empty() {
            return None;
        }

        let row = &result.unwrap()[0];
        let slot: i64 = row.get(0);

        Some(slot as u64)
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
        let mut cur_slot = 1;
        loop {
            let shreds = self.load_shred_from_pg(cur_slot);
            shreds.into_iter().for_each(|s| {
                let res = self
                    .blockstore
                    .as_mut()
                    .unwrap()
                    .insert_shreds(vec![s], None, false);
                if res.is_err() {
                    println!("insert failed at slot: {}", cur_slot);
                }
            });

            if cur_slot >= slot {
                break;
            }
            cur_slot += 1;
        }
        Ok(())
    }

    pub fn insert_shred_startwith_slot(&mut self, slot: u64) -> Result<(), ReplayerError> {
        let mut verified: i64 = (slot - 1) as i64;
        let mut cur_slot: i64 = slot as i64;
        let ledger_path = self
            .ledger_path
            .as_ref()
            .unwrap()
            .as_path()
            .display()
            .to_string();
        let entry_index = 0_i64;
        loop {
            let mut flag = true;
            let shreds = self.load_shred_from_pg(cur_slot as u64);
            if shreds.is_empty() {
                // no more new shred available
                println!(
                    "[{:?}]No more new shred available at slot {} ",
                    chrono::offset::Utc::now(),
                    cur_slot
                );

                // in case cur_slot is restart point, we try next slot.
                let shreds = self.load_shred_from_pg((cur_slot + 1) as u64);
                if !shreds.is_empty() {
                    cur_slot += 1;

                    continue;
                }
                std::thread::sleep(std::time::Duration::from_secs(10));
                continue;
            }
            shreds.into_iter().for_each(|s| {
                let res = self
                    .blockstore
                    .as_mut()
                    .unwrap()
                    .insert_shreds(vec![s], None, false);
                if res.is_err() {
                    println!("insert shred failed at slot: {}", cur_slot);
                    flag = false;
                }
            });

            if !flag {
                break;
            }

            // Every VERIFY_INTERVAL_SLOTS slot we do a create-snapshot and verify.
            if cur_slot == verified + Self::VERIFY_INTERVAL_SLOTS {
                // verify first
                let v_out = run_ledger_tool(&["-l", &ledger_path, "verify", "--halt-at-slot", &cur_slot.to_string(),]);
                if !v_out.status.success() {
                    println!("verify replay ledger failed at slot: {}", cur_slot);
                    println!("{:?}", v_out);
                    break;
                }

                println!("verify replay ledger succed at slot: {}", cur_slot);
                // then create snapshot
                let cs_slot = cur_slot - (Self::VERIFY_INTERVAL_SLOTS - Self::CREATE_SNAPSHOT_INTERVAL_SLOTS);
                let cs_out = run_ledger_tool(&[
                    "-l",
                    &ledger_path,
                    "create-snapshot",
                    &cs_slot.to_string(),
                    &ledger_path,
                ]);
                if !cs_out.status.success() {
                    println!("create snapshot failed for slot: {}", cs_slot);
                    println!("{:?}", cs_out);
                    break;
                }
                println!("create snapshot succed for slot: {}", cs_slot);

                // all good? save slot in record db
                let client = self.client.as_mut().unwrap();
                client
                    .execute(
                        "INSERT INTO replay (slot, entry_index) VALUES ($1, $2)",
                        &[&cur_slot, &entry_index],
                    )
                    .unwrap();

                verified = cur_slot;
            }
            cur_slot += 1;
        }

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
