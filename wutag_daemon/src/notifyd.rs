use crate::registry::try_get_registry_write_loop;
use crate::{EntryEvent, Error, Result, ENTRIES_EVENTS, NOTIFY_EVENTS};
use notify::{
    self, event::RemoveKind, Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher,
};
use std::mem;
use std::path::Path;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum NotifyDaemonError {
    #[error("failed to initialize notify watcher - {0}")]
    NotifyWatcherInit(notify::Error),
    #[error("failed to add watch entry - {0}")]
    AddWatchEntry(notify::Error),
    #[error("failed to remove watch entry - {0}")]
    RemoveWatchEntry(notify::Error),
}

pub struct NotifyDaemon {
    notify: RecommendedWatcher,
}

struct Handler;

impl EventHandler for Handler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        match event {
            Ok(event) => match event.kind {
                EventKind::Remove(RemoveKind::File)
                | EventKind::Remove(RemoveKind::Any)
                | EventKind::Remove(RemoveKind::Folder)
                | EventKind::Remove(RemoveKind::Other) => match NOTIFY_EVENTS.try_write() {
                    Ok(mut events) => events.push(event),
                    Err(e) => log::error!("failed to lock notify events, reason: {e}"),
                },
                _ => {}
            },
            Err(e) => {
                log::error!("failed to read notify event, reason: {e}");
            }
        }
    }
}

impl NotifyDaemon {
    pub fn new() -> Result<Self> {
        let mut d = Self {
            notify: RecommendedWatcher::new(Handler, Default::default())
                .map_err(NotifyDaemonError::NotifyWatcherInit)?,
        };

        d.rebuild_watch_entries().map(|_| d)
    }

    pub fn work_loop(mut self) {
        loop {
            if let Err(e) = self.handle_entries_events() {
                log::error!("{e}");
            }
            if let Err(e) = self.handle_notify_events() {
                log::error!("{e}");
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    fn rebuild_watch_entries(&mut self) -> Result<()> {
        let mut registry = try_get_registry_write_loop()?;
        let mut to_remove = vec![];
        for entry in registry.list_entries().cloned() {
            if let Err(e) = self.add_watch_entry(entry.path()) {
                log::error!("{e}");
                match e {
                    crate::Error::NotifyDaemon(NotifyDaemonError::NotifyWatcherInit(e)) => {
                        if let notify::ErrorKind::Io(err) = &e.kind {
                            if let std::io::ErrorKind::NotFound = err.kind() {
                                to_remove.push(entry);
                            }
                        }
                    }
                    _ => {}
                }
                continue;
            }
        }
        for entry in to_remove {
            log::info!(
                "entry `{}` not found, removing from registry",
                entry.path().display()
            );
            if let Some(id) = registry.find_entry(entry.path()) {
                registry.remove_entry(id);
            }
        }
        registry.save().map_err(Error::RegistrySave)?;
        Ok(())
    }

    fn add_watch_entry(&mut self, entry: impl AsRef<Path>) -> Result<()> {
        let entry = entry.as_ref();
        log::trace!("adding watch entry {}", entry.display());
        self.notify
            .watch(entry, RecursiveMode::NonRecursive)
            .map_err(NotifyDaemonError::AddWatchEntry)
            .map_err(Error::from)
    }

    fn remove_watch_entry(&mut self, entry: impl AsRef<Path>) -> Result<()> {
        let entry = entry.as_ref();
        log::trace!("removing watch entry {}", entry.display());
        self.notify
            .unwatch(entry)
            .map_err(NotifyDaemonError::RemoveWatchEntry)
            .map_err(Error::from)
    }

    fn handle_notify_events(&mut self) -> Result<()> {
        let mut events_handle = match NOTIFY_EVENTS.try_write() {
            Ok(events) => events,
            Err(e) => {
                return Err(Error::NotifyEventsLock(e.to_string()));
            }
        };
        if events_handle.is_empty() {
            return Ok(());
        }
        let events = mem::take(&mut *events_handle);
        mem::drop(events_handle);
        let mut registry = try_get_registry_write_loop()?;
        for event in events {
            for path in event.paths {
                if let Some(id) = registry.find_entry(&path) {
                    log::trace!("removing entry {}, id: {id}", path.display());
                    registry.clear_entry(id);
                }
            }
        }
        registry.save().map_err(Error::RegistrySave)?;
        Ok(())
    }

    fn handle_entries_events(&mut self) -> Result<()> {
        let mut events_handle = match ENTRIES_EVENTS.try_write() {
            Ok(events) => events,
            Err(e) => {
                return Err(Error::EntriesEventsLock(e.to_string()));
            }
        };
        if events_handle.is_empty() {
            return Ok(());
        }
        let events = mem::take(&mut *events_handle);
        mem::drop(events_handle);

        for event in events {
            log::trace!("handling entry event {event:?}");
            match event {
                EntryEvent::Add(entries) => {
                    for entry in entries {
                        if let Err(e) = self.add_watch_entry(entry) {
                            log::error!("{e}");
                            continue;
                        }
                    }
                }
                EntryEvent::Remove(entries) => {
                    for entry in entries {
                        if let Err(e) = self.remove_watch_entry(&entry) {
                            log::error!("{}: {e}", entry.display());
                            continue;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
