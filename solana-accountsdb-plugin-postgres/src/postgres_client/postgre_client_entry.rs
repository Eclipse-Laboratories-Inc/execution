
// build_entry_upsert_statement

use {
    crate::{
        geyser_plugin_postgres::{GeyserPluginPostgresConfig, GeyserPluginPostgresError},
        postgres_client::{
            SimplePostgresClient, LogEntryRequest
        },
    },
    chrono::Utc,
    log::*,
    postgres::{Client, Statement},
    solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError
};

impl SimplePostgresClient {
    pub(crate) fn build_entry_upsert_statement(
        client: &mut Client,
        config: &GeyserPluginPostgresConfig,
    ) -> Result<Statement, GeyserPluginError> {
        let stmt =
            "INSERT INTO entry (entry, updated_on) \
        VALUES ($1, $2)";

        let stmt = client.prepare(stmt);

        match stmt {
            Err(err) => {
                Err(GeyserPluginError::Custom(Box::new(GeyserPluginPostgresError::DataSchemaError {
                    msg: format!(
                        "Error in preparing for the entry update PostgreSQL database: ({}) host: {:?} user: {:?} config: {:?}",
                        err, config.host, config.user, config
                    ),
                })))
            }
            Ok(stmt) => Ok(stmt),
        }
    }

    pub(crate) fn log_entry_impl(
        &mut self,
        entry: LogEntryRequest,
    ) -> Result<(), GeyserPluginError> {
        let client = self.client.get_mut().unwrap();
        let statement = &client.log_entry_stmt;
        let client = &mut client.client;
        let updated_on = Utc::now().naive_utc();
        let bin:&[u8] = &vec![1,2,3] ;
        let result = client.execute(
            statement,
           &[&bin, &updated_on]
        );

        if let Err(err) = result {
            let msg = format!(
                "Failed to persist entry/shred to the PostgreSQL database. Error: {:?}",
                err);
            error!("{}", msg);
            return Err(GeyserPluginError::EntryUpdateError {msg});
        }

        Ok(())
    }
}
