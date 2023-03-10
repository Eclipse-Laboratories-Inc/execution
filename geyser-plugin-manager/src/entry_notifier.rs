use {
    crate::{entry_notifier_interface::EntryNotifier, geyser_plugin_manager::GeyserPluginManager},
    log::*,
    solana_entry::entry::UntrustedEntry,
    // solana_measure::measure::Measure,
    // solana_metrics::*,
    std::sync::{Arc, RwLock},
};

pub(crate) struct EntryNotifierImpl {
    plugin_manager: Arc<RwLock<GeyserPluginManager>>,
}

impl EntryNotifier for EntryNotifierImpl {
    /// Notify the entry
    fn notify_entry(&self, entry: &UntrustedEntry) {
        info!("notify_entry: entries: {}, slot: {}, parent_slot: {}, is_full_slot: {}", entry.entries.len(), entry.slot, entry.parent_slot, entry.is_full_slot);
        if !entry.is_full_slot {
            let a = 0;
            println!("error");
        }
        let mut plugin_manager = self.plugin_manager.write().unwrap();
        if plugin_manager.plugins.is_empty() {
            return;
        }
        for plugin in plugin_manager.plugins.iter_mut() {
            if !plugin.entry_notifications_enabled() {
                continue;
            }
            match plugin.notify_entry(entry) {
                Err(err) => {
                    error!(
                        "Failed to update shred error: {:?} to plugin {}",
                        err,
                        plugin.name()
                    )
                }
                Ok(_) => {
                    trace!("Successfully updated shred to plugin {}", plugin.name());
                }
            }
        }
    }

    fn last_insert_entry(&self) -> u64 {
        let mut plugin_manager = self.plugin_manager.write().unwrap();
        if plugin_manager.plugins.is_empty() {
            return 0;
        }

        for plugin in plugin_manager.plugins.iter_mut() {
            if !plugin.entry_notifications_enabled() {
                continue;
            }
            return plugin.last_insert_entry();
        }

        0
    }
}

impl EntryNotifierImpl {
    pub fn new(plugin_manager: Arc<RwLock<GeyserPluginManager>>) -> Self {
        Self { plugin_manager }
    }
}
