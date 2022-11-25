use {
    crate::{
        geyser_plugin_manager::GeyserPluginManager, entry_notifier_interface::EntryNotifier
    },
    log::*,
    solana_entry::entry::Entry,
    solana_measure::measure::Measure,
    solana_metrics::*,
    std::sync::{Arc, RwLock},
};

pub(crate) struct EntryNotifierImpl {
    plugin_manager: Arc<RwLock<GeyserPluginManager>>,
}

impl EntryNotifier for EntryNotifierImpl {
    /// Notify the entry
    fn notify_entry(&self, entry: &Entry) {
        let mut plugin_manager = self.plugin_manager.write().unwrap();
        if plugin_manager.plugins.is_empty() {
            return;
        }
        for plugin in plugin_manager.plugins.iter_mut() {
            match plugin.notify_entry() {
                Err(err) => {
                    error!(
                        "Failed to update shred error: {:?} to plugin {}",
                        err,
                        plugin.name()
                    )
                }
                Ok(_) => {
                    trace!(
                        "Successfully updated shred to plugin {}",
                        plugin.name()
                    );
                }
            }
        }

    }
}

impl EntryNotifierImpl {
    pub fn new(plugin_manager: Arc<RwLock<GeyserPluginManager>>) -> Self {
        Self { plugin_manager }
    }

    fn build_shred(
        entry: &Entry,
    ) {
    }
}
