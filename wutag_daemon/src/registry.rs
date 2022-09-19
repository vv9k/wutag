use anyhow::{Error, Result};
use once_cell::sync::Lazy;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};
use wutag_core::registry::TagRegistry;

static REGISTRY: Lazy<RwLock<TagRegistry>> = Lazy::new(|| {
    let registry_file = dirs::data_dir()
        .expect("valid data directory")
        .join("wutag.db");
    RwLock::new(
        TagRegistry::load(&registry_file).unwrap_or_else(|_| TagRegistry::new(registry_file)),
    )
});

pub fn get_registry_write() -> RwLockWriteGuard<'static, TagRegistry> {
    match REGISTRY.try_write() {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("failed to lock registry for writing, reason: {e}");
            std::process::exit(1);
        }
    }
}
pub fn get_registry_read() -> RwLockReadGuard<'static, TagRegistry> {
    match REGISTRY.try_read() {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("failed to lock registry for reading, reason: {e}");
            std::process::exit(1);
        }
    }
}

pub fn try_get_registry_write_loop() -> Result<RwLockWriteGuard<'static, TagRegistry>> {
    let mut i = 0;
    loop {
        i += 1;
        if i >= 5 {
            return Err(Error::msg("failed to lock registry for writing"));
        }
        let registry = match REGISTRY.try_write() {
            Ok(registry) => registry,
            Err(e) => match e {
                TryLockError::Poisoned(e) => {
                    return Err(Error::msg(format!(
                        "failed to lock registry for writing, reason: {e}"
                    )))
                }
                TryLockError::WouldBlock => continue,
            },
        };
        break Ok(registry);
    }
}
