use {
    crossbeam_channel::{Receiver, RecvTimeoutError},
    solana_ledger::blockstore::Blockstore,
    solana_measure::measure::Measure,
    std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread::{self, Builder, JoinHandle},
        time::Duration,
    },
    solana_entry::entry::EntryReceiver,
};
use solana_geyser_plugin_manager::entry_notifier_interface::EntryNotifierLock;

pub struct EntryService {
    thread_hdl: JoinHandle<()>,
}

impl EntryService {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        entry_receiver: EntryReceiver,
        entry_notifier: Option<EntryNotifierLock>,
        exit: &Arc<AtomicBool>,
    ) -> Self {
        let exit = exit.clone();
        let thread_hdl = Builder::new()
            .name("solEntryService".to_string())
            .spawn(move || loop {
                if exit.load(Ordering::Relaxed) {
                    break;
                }
                let recv_result = entry_receiver.recv_timeout(Duration::from_secs(1));
                match recv_result {
                    Err(RecvTimeoutError::Disconnected) => {
                        debug!("EntryService recv fail");
                        break;
                    }
                    Ok(entries) => {
                        if let Some(entry_notifier) = entry_notifier.as_ref() {
                            debug!("EntryService recv succ {:?}", &entries);
                            for entry in entries.iter() {
                                let a = entry_notifier.write().unwrap();
                                a.notify_entry();
                                entry_notifier.write().unwrap().notify_entry(entry);
                            }
                        }
                    }
                    _ => {}
                }
            })
            .unwrap();
        Self { thread_hdl }
    }

    pub fn join(self) -> thread::Result<()> {
        self.thread_hdl.join()
    }
}
