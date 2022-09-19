mod daemon;
mod notifyd;
mod registry;

use daemon::WutagDaemon;
use notifyd::NotifyDaemon;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::RwLock;
use thiserror::Error as ThisError;
use wutag_ipc::{default_socket, IpcServer};

pub static ENTRIES_EVENTS: Lazy<RwLock<Vec<EntryEvent>>> = Lazy::new(|| RwLock::new(Vec::new()));
pub static NOTIFY_EVENTS: Lazy<RwLock<Vec<notify::Event>>> = Lazy::new(|| RwLock::new(Vec::new()));

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Registry(#[from] registry::RegistryError),
    #[error(transparent)]
    NotifyDaemon(#[from] notifyd::NotifyDaemonError),
    #[error(transparent)]
    Daemon(#[from] daemon::DaemonError),
    #[error(transparent)]
    RegistrySave(wutag_core::registry::RegistryError),
    #[error("failed to lock notify events - {0}")]
    NotifyEventsLock(String),
    #[error("failed to lock entries events - {0}")]
    EntriesEventsLock(String),
    #[error(transparent)]
    IpcServerInit(wutag_ipc::IpcError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum EntryEvent {
    Add(Vec<PathBuf>),
    Remove(Vec<PathBuf>),
}

pub fn main() -> Result<()> {
    pretty_env_logger::init();

    let listener = IpcServer::new(default_socket()).map_err(Error::IpcServerInit)?;
    let mut daemon = WutagDaemon::new(listener)?;
    let mut notify_daemon = NotifyDaemon::new()?;
    notify_daemon.rebuild_watch_descriptors()?;

    std::thread::scope(|s| {
        let h1 = s.spawn(|| loop {
            if let Err(e) = daemon.process_connection() {
                log::error!("Failed to process connection, reason: '{e}'");
            }
        });
        let h2 = s.spawn(|| {
            notify_daemon.work_loop();
        });

        h1.join().unwrap();
        h2.join().unwrap();
    });

    Ok(())
}
