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

pub struct EntryService {
    thread_hdl: JoinHandle<()>,
}

impl EntryService {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        entry_receiver: EntryReceiver,
        blockstore: Arc<Blockstore>,
        exit: &Arc<AtomicBool>,
    ) -> Self {
        let exit = exit.clone();
        let thread_hdl = Builder::new()
            .name("solCacheBlkTime".to_string())
            .spawn(move || loop {
                if exit.load(Ordering::Relaxed) {
                    break;
                }
                let recv_result = entry_receiver.recv_timeout(Duration::from_secs(1));
                match recv_result {
                    Err(RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    Ok(entries) => {
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
