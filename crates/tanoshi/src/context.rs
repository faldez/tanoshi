use crate::db::{MangaDatabase, UserDatabase};
use crate::worker::Command as WorkerCommand;
use tanoshi_vm::bus::ExtensionBus;
use tokio::sync::mpsc::UnboundedSender;

pub struct GlobalContext {
    pub userdb: UserDatabase,
    pub secret: String,
    pub mangadb: MangaDatabase,
    pub extensions: ExtensionBus,
    pub worker_tx: UnboundedSender<WorkerCommand>,
}

impl GlobalContext {
    pub fn new(
        userdb: UserDatabase,
        mangadb: MangaDatabase,
        secret: String,
        extensions: ExtensionBus,
        worker_tx: UnboundedSender<WorkerCommand>,
    ) -> Self {
        Self {
            userdb,
            secret,
            mangadb,
            extensions,
            worker_tx,
        }
    }
}
