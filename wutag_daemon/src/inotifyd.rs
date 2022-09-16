use crate::registry::{try_get_registry_read_loop, try_get_registry_write_loop};
use crate::{EntryEvent, ENTRIES_EVENTS};
use anyhow::{Context, Error, Result};
use inotify::{Event, EventMask, Inotify, WatchDescriptor, WatchMask};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub struct InotifyDaemon {
    watch_descriptors: HashMap<WatchDescriptor, PathBuf>,
    inotify: Inotify,
}

impl InotifyDaemon {
    pub fn new() -> Result<Self> {
        Ok(Self {
            watch_descriptors: HashMap::new(),
            inotify: Inotify::init().expect("failed to initialize inotify"),
        })
    }

    pub fn rebuild_watch_descriptors(&mut self) -> Result<()> {
        let registry = try_get_registry_read_loop()?;
        for entry in registry.list_entries().cloned() {
            if let Err(e) = self.add_watch_entry(entry.path()) {
                log::error!("{e:?}");
                continue;
            }
        }
        Ok(())
    }

    fn add_watch_entry(&mut self, entry: impl AsRef<Path>) -> Result<()> {
        let entry = entry.as_ref();
        log::trace!("adding watch entry {}", entry.display());
        let wd = self
            .inotify
            .add_watch(entry, WatchMask::DELETE_SELF | WatchMask::MOVE_SELF)
            .context(format!(
                "failed to add watch descriptor for `{}`",
                entry.display()
            ))?;
        self.watch_descriptors.insert(wd, entry.to_path_buf());
        Ok(())
    }

    fn remove_watch_entry(&mut self, entry: impl AsRef<Path>) -> Option<()> {
        let entry = entry.as_ref();
        log::trace!("removing watch entry {}", entry.display());
        let k = self
            .watch_descriptors
            .iter()
            .find(|(_, p)| p.as_path() == entry)
            .map(|(k, _)| k.to_owned())?;
        self.watch_descriptors.remove(&k).map(|_| ())
    }

    pub fn work_loop(mut self) {
        loop {
            let mut buf = [0; 1024];
            if let Err(e) = self.handle_inotify_events(&mut buf) {
                log::error!("{e:?}");
            }
            if let Err(e) = self.handle_entries_events() {
                log::error!("{e:?}");
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    fn handle_entries_events(&mut self) -> Result<()> {
        let mut events = match ENTRIES_EVENTS.try_write() {
            Ok(events) => events,
            Err(e) => {
                return Err(Error::msg(format!(
                    "failed to lock entries events, reason: {e}"
                )))
            }
        };
        if events.is_empty() {
            return Ok(());
        }
        let events = std::mem::take(&mut *events);
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
                        if self.remove_watch_entry(&entry).is_none() {
                            log::error!(
                                "watch descriptor not found for entry `{}`",
                                entry.display()
                            );
                            continue;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_inotify_events(&mut self, buf: &mut [u8; 1024]) -> Result<()> {
        let events = match self.inotify.read_events(buf) {
            Ok(events) => events,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => {
                return Err(Error::msg(format!(
                    "error while reading event, reason: {e}"
                )))
            }
        };
        for event in events {
            if let Err(e) = self.handle_event(event) {
                log::error!("error while handling event, reason: {e}");
                continue;
            };
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event<&OsStr>) -> Result<()> {
        log::trace!("{event:?}");
        if event.mask.contains(EventMask::MOVE_SELF) || event.mask.contains(EventMask::DELETE_SELF)
        {
            let path = self
                .watch_descriptors
                .remove(&event.wd)
                .context("failed to match watch descriptor to an entry")?;
            if let Err(e) = self.inotify.rm_watch(event.wd) {
                log::error!(
                    "failed to remove watch descriptor for {}, reason: {e}",
                    path.display()
                );
            }
            if self.remove_watch_entry(&path).is_none() {
                log::error!("watch descriptor not found for entry `{}`", path.display());
            }
            let mut registry = try_get_registry_write_loop()?;
            registry
                .find_entry(&path)
                .and_then(|id| registry.remove_entry(id))
                .ok_or_else(|| {
                    Error::msg(format!(
                        "failed to find entry `{}` in registry",
                        path.display()
                    ))
                })?;
        }
        Ok(())
    }
}
