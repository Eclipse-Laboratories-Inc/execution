use {
    thiserror::Error,
    postgres::{Client, NoTls},
    std::{io, path::PathBuf},
    serde_derive::{Deserialize, Serialize},
};

#[derive(Default)]
pub struct Replayer {
    client: Option<Client>,
}

impl Replayer {
    pub fn new(config_file: ReplayerPostgresConfig, ledger_path: PathBuf) -> Replayer {
        Replayer { client: None }
    }

    pub fn todo(&mut self) {
        /*
        let connection_str = format!(
            "host={} user={} password={} dbname={} port={}",
            config.host.as_ref().unwrap(),
            config.user.as_ref().unwrap(),
            config.password.as_ref().unwrap(),
            config.dbname.as_ref().unwrap(),
            config.port.as_ref().unwrap(),
        );

        let mut client = Client::connect(&connection_str, NoTls).unwrap();
        let stmt = "SELECT slot, entry_index, entry FROM entry where slot = $1";
        let stmt = client.prepare(stmt).unwrap();

        let blockstore = Blockstore::open(ledger_path.as_path()).unwrap();
        let slot: i64 = 1;

        let shreds: Vec<Shred> = Vec::new();
        // Query shred by slot and update blockstore.
        let result = client.query(&stmt, &[&slot]);
        if let Err(e) = result {

        }
        for row in res.unwrap() {
            let payload: Vec<u8> = row.get(0);
            let result = Shred::new_from_serialized_shred(payload);
            if let Err(e) = result {

            }
            shreds.append(result.unwrap())
        };

        for row in client.query(&stmt, &[&slot]).unwrap() {
            let mut success = false;
            while !success {
                match blockstore.insert_shreds(vec![shred.clone()], None, false) {
                    Ok(_) => success = true,
                    Err(_) => success = false,
                }
            }
        }
         */
    }
}

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
