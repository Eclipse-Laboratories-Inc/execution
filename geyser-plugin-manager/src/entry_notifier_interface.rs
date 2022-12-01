use {
    solana_entry::entry::Entry,
    std::sync::{Arc, RwLock},
};

/// Interface for notifying block entry changes
pub trait EntryNotifier {
    /// Notify the entry
    fn notify_entry(&self, entry: &Entry);
}

pub type EntryNotifierLock = Arc<RwLock<dyn EntryNotifier + Sync + Send>>;
