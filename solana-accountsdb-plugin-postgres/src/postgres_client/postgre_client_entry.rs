// build_entry_upsert_statement

use {
    crate::{
        geyser_plugin_postgres::{GeyserPluginPostgresConfig, GeyserPluginPostgresError},
        postgres_client::{LogEntryRequest, SimplePostgresClient},
    },
    chrono::Utc,
    log::*,
    postgres::{Client, Statement},
    solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError,
    solana_ledger::blockstore,
};

impl SimplePostgresClient {
    pub(crate) fn build_entry_upsert_statement(
        client: &mut Client,
        config: &GeyserPluginPostgresConfig,
    ) -> Result<Statement, GeyserPluginError> {
        let stmt =
            "INSERT INTO entry (slot, parent_slot, entry_index, entry, is_full_slot, updated_on) \
        VALUES ($1, $2, $3, $4, $5, $6)";

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
        log_entry_request: LogEntryRequest,
    ) -> Result<(), GeyserPluginError> {
        let client = self.client.get_mut().unwrap();
        let statement = &client.log_entry_stmt;
        let client = &mut client.client;

        let updated_on = Utc::now().naive_utc();

        // entry to shred, 64 entry ~= 8 shred
        let entry = &log_entry_request.entry;
        let entries = &entry.entries;

        let (slot, parent_slot, is_full_slot) = (entry.slot, entry.parent_slot, entry.is_full_slot);
        let (version, merkle_variant) = (0, true);

        let shreds = blockstore::entries_to_test_shreds(
            &entries,
            slot,
            parent_slot,
            is_full_slot,
            version,
            merkle_variant,
        );

        for (index, shred) in shreds.iter().enumerate() {
            let result = client.execute(
                statement,
                &[
                    &(slot as i64),
                    &(parent_slot as i64),
                    &(index as i64),
                    &shred.payload(),
                    &is_full_slot,
                    &updated_on,
                ],
            );
            if let Err(err) = result {
                let msg = format!(
                    "Failed to persist entry/shred to the PostgreSQL database. Error: {:?}",
                    err
                );
                error!("{}", msg);
                return Err(GeyserPluginError::EntryUpdateError { msg });
            }
        }

        Ok(())
    }
}
