use {
    std::sync::{Arc, RwLock},
    solana_entry::entry::Entry,
};

/// Interface for notifying block entry changes
pub trait EntryNotifier {
    /// Notify the entry
    fn notify_entry(
        &self,
        entry: &Entry,
    );
}

pub type EntryNotifierLock = Arc<RwLock<dyn EntryNotifier + Sync + Send>>;
