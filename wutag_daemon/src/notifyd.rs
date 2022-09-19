use crate::registry::try_get_registry_write_loop;
use crate::{EntryEvent, ENTRIES_EVENTS, NOTIFY_EVENTS};
use anyhow::{Context, Error, Result};
use notify::{
    self, event::RemoveKind, Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher,
};
use std::mem;
use std::path::Path;

pub struct NotifyDaemon {
    notify: RecommendedWatcher,
}

struct Handler;

impl<'a> EventHandler for Handler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        match event {
            Ok(event) => match event.kind {
                EventKind::Remove(RemoveKind::File)
                | EventKind::Remove(RemoveKind::Any)
                | EventKind::Remove(RemoveKind::Folder)
                | EventKind::Remove(RemoveKind::Other) => match NOTIFY_EVENTS.try_write() {
                    Ok(mut events) => events.push_back(event),
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
        Ok(Self {
            notify: RecommendedWatcher::new(Handler, Default::default())
                .context("failed to initialize notify watcher")?,
        })
    }

    pub fn rebuild_watch_descriptors(&mut self) -> Result<()> {
        let mut registry = try_get_registry_write_loop()?;
        let mut to_remove = vec![];
        for entry in registry.list_entries().cloned() {
            if let Err(e) = self.add_watch_entry(entry.path()) {
                log::error!("{e:?}");
                if let Some(err) = e
                    .source()
                    .and_then(|src| src.downcast_ref::<notify::Error>())
                {
                    if let notify::ErrorKind::Io(err) = &err.kind {
                        if let std::io::ErrorKind::NotFound = err.kind() {
                            to_remove.push(entry);
                        }
                    }
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
        registry.save()?;
        Ok(())
    }

    fn add_watch_entry(&mut self, entry: impl AsRef<Path>) -> Result<()> {
        let entry = entry.as_ref();
        log::trace!("adding watch entry {}", entry.display());
        self.notify
            .watch(entry, RecursiveMode::NonRecursive)
            .context("failed to watch entry")
    }

    fn remove_watch_entry(&mut self, entry: impl AsRef<Path>) -> Result<()> {
        let entry = entry.as_ref();
        log::trace!("removing watch entry {}", entry.display());
        self.notify
            .unwatch(entry)
            .context("failed to unwatch entry")
    }

    pub fn work_loop(mut self) {
        loop {
            if let Err(e) = self.handle_entries_events() {
                log::error!("{e:?}");
            }
            if let Err(e) = self.handle_notify_events() {
                log::error!("{e:?}");
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    fn handle_notify_events(&mut self) -> Result<()> {
        let mut events_handle = match NOTIFY_EVENTS.try_write() {
            Ok(events) => events,
            Err(e) => {
                return Err(Error::msg(format!(
                    "failed to lock notify events, reason: {e}"
                )))
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
        registry.save()?;
        Ok(())
    }

    fn handle_entries_events(&mut self) -> Result<()> {
        let mut events_handle = match ENTRIES_EVENTS.try_write() {
            Ok(events) => events,
            Err(e) => {
                return Err(Error::msg(format!(
                    "failed to lock entries events, reason: {e}"
                )))
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
                            log::error!("{e:?}");
                            continue;
                        }
                    }
                }
                EntryEvent::Remove(entries) => {
                    for entry in entries {
                        if let Err(e) = self.remove_watch_entry(&entry) {
                            log::error!("{}: {e:?}", entry.display());
                            continue;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
