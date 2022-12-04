use {
    solana_entry::entry::UntrustedEntry,
    std::sync::{Arc, RwLock},
};

/// Interface for notifying block entry changes
pub trait EntryNotifier {
    /// Notify the entry
    fn notify_entry(&self, entry: &UntrustedEntry);
}

pub type EntryNotifierLock = Arc<RwLock<dyn EntryNotifier + Sync + Send>>;
