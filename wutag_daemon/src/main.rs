mod daemon;
mod notifyd;
mod registry;

use anyhow::Context;
use daemon::WutagDaemon;
use notifyd::NotifyDaemon;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::RwLock;
use wutag_ipc::{default_socket, IpcServer};

pub static ENTRIES_EVENTS: Lazy<RwLock<Vec<EntryEvent>>> = Lazy::new(|| RwLock::new(Vec::new()));
pub static NOTIFY_EVENTS: Lazy<RwLock<Vec<notify::Event>>> = Lazy::new(|| RwLock::new(Vec::new()));

#[derive(Debug)]
pub enum EntryEvent {
    Add(Vec<PathBuf>),
    Remove(Vec<PathBuf>),
}

pub fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let listener = IpcServer::new(default_socket())?;
    let mut daemon = WutagDaemon::new(listener).context("failed to initialize daemon")?;
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
