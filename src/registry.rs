#![allow(dead_code)]
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use wutag_core::tags::Tag;
use wutag_core::{Error, Result};

use colored::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct EntryData {
    path: PathBuf,
}

impl EntryData {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub type EntryId = usize;

#[derive(Default, Deserialize, Serialize)]
pub struct TagRegistry {
    tags: HashMap<Tag, Vec<EntryId>>,
    entries: HashMap<EntryId, EntryData>,
    path: PathBuf,
}

impl TagRegistry {
    /// Loads a registry from the specified `path`.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let data = fs::read(path)?;

        serde_cbor::from_slice(&data).map_err(|e| Error::Other(e.to_string()))
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Saves the registry serialized to the path from which it was loaded.
    pub fn save(&self) -> Result<()> {
        let serialized = serde_cbor::to_vec(&self)?;
        fs::write(&self.path, &serialized).map_err(|e| Error::Other(e.to_string()))
    }

    /// Updates the entry or adds it if it is not present.
    pub fn add_or_update_entry(&mut self, entry: EntryData) -> EntryId {
        let pos = self
            .list_entries_and_ids()
            .find(|(_, e)| **e == entry)
            .map(|(idx, _)| *idx);

        let pos = if let Some(pos) = pos {
            let e = self.entries.get_mut(&pos).unwrap();
            *e = entry;
            pos
        } else {
            let timestamp = chrono::Utc::now().timestamp_nanos();
            let timestamp = if timestamp < 0 {
                timestamp.abs() as usize
            } else {
                timestamp as usize
            };
            self.entries.insert(timestamp, entry);
            timestamp
        };

        pos
    }

    fn mut_tag_entries(&mut self, tag: &Tag) -> &mut Vec<EntryId> {
        let exists = self.tags.iter().find(|(t, _)| t == &tag);

        if exists.is_none() {
            self.tags.insert(tag.clone(), Vec::new());
        }

        self.tags.get_mut(tag).unwrap()
    }

    /// Adds the `tag` to an entry with `entry` id. Returns the id if the entry was already tagged
    /// or `None` if the tag was added.
    pub fn tag_entry(&mut self, tag: &Tag, entry: EntryId) -> Option<EntryId> {
        let entries = self.mut_tag_entries(tag);

        if let Some(entry) = entries.iter().find(|&e| *e == entry) {
            return Some(*entry);
        }
        entries.push(entry);

        None
    }

    /// Removes the `tag` from an entry with `entry` id. Returns the id if the tag was removed or
    /// `None` if the tag was not present.
    pub fn untag_entry(&mut self, tag: &Tag, entry: EntryId) -> Option<EntryId> {
        let entries = self.mut_tag_entries(tag);

        if let Some(pos) = entries.iter().position(|e| *e == entry) {
            return Some(entries.remove(pos));
        }

        None
    }

    /// Removes the tag with the `name` from the `entry`.
    pub fn untag_by_name(&mut self, tag: &str, entry: EntryId) -> Option<EntryId> {
        let tag = self.get_tag(tag)?.to_owned();
        let entries = self.mut_tag_entries(&tag);

        if let Some(pos) = entries.iter().position(|e| *e == entry) {
            return Some(entries.remove(pos));
        }

        None
    }

    /// Clears all tags of the `entry`.
    pub fn clear_entry(&mut self, entry: EntryId) {
        self.tags.iter_mut().for_each(|(_, entries)| {
            if let Some(idx) = entries.iter().copied().position(|e| e == entry) {
                entries.remove(idx);
            }
        })
    }

    /// Finds the entry by a `path`. Returns the id of the entry.
    pub fn find_entry<P: AsRef<Path>>(&self, path: P) -> Option<EntryId> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.path == path.as_ref())
            .map(|(idx, _)| *idx)
    }

    /// Lists tags of an entry if such entry exists
    pub fn list_entry_tags(&self, entry: EntryId) -> Option<Vec<&Tag>> {
        let tags = self
            .tags
            .iter()
            .fold(Vec::new(), |mut acc, (tag, entries)| {
                if entries.iter().any(|id| entry == *id) {
                    acc.push(tag);
                }
                acc
            });

        if tags.is_empty() {
            None
        } else {
            Some(tags)
        }
    }

    pub fn list_entries_with_tags<T, S>(&self, tags: T) -> Vec<EntryId>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut entries = tags.into_iter().fold(Vec::new(), |mut acc, tag| {
            if let Some(entries) = self
                .tags
                .iter()
                .find(|(t, _)| t.name() == tag.as_ref())
                .map(|(_, e)| e)
            {
                acc.extend_from_slice(&entries[..]);
            }
            acc
        });

        entries.dedup();

        entries
    }

    pub fn list_entries_ids(&self) -> impl Iterator<Item = &EntryId> {
        self.entries.keys()
    }

    pub fn list_entries(&self) -> impl Iterator<Item = &EntryData> {
        self.entries.values()
    }

    pub fn list_entries_and_ids(&self) -> impl Iterator<Item = (&EntryId, &EntryData)> {
        self.entries.iter()
    }

    pub fn list_tags(&self) -> impl Iterator<Item = &Tag> {
        self.tags.keys()
    }

    pub fn get_entry(&self, id: EntryId) -> Option<&EntryData> {
        self.entries.get(&id)
    }

    pub fn get_tag<T: AsRef<str>>(&self, tag: T) -> Option<&Tag> {
        self.tags.keys().find(|t| t.name() == tag.as_ref())
    }

    pub fn update_tag_color<T: AsRef<str>>(&mut self, tag: T, color: Color) -> bool {
        if let Some(mut t) = self.tags.keys().find(|t| t.name() == tag.as_ref()).cloned() {
            let data = self.tags.remove(&t).unwrap();
            t.set_color(&color);
            self.tags.insert(t, data);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_COLORS;
    use colored::Color::{Black, Red};

    #[test]
    fn adds_and_tags_entry() {
        let path = PathBuf::from("/tmp");
        let entry = EntryData { path: path.clone() };
        let mut registry = TagRegistry::default();
        registry.add_or_update_entry(entry.clone());
        let id = registry.find_entry(&path).unwrap();

        let _entry = registry.get_entry(id).unwrap();
        assert_eq!(_entry.path, entry.path);

        let tag = Tag::random("test", DEFAULT_COLORS);
        let second = Tag::random("second", DEFAULT_COLORS);

        assert_eq!(registry.tag_entry(&tag, id), None);
        assert_eq!(registry.list_entry_tags(id), Some(vec![&tag]));
        assert_eq!(registry.tag_entry(&second, id), None);
        assert!(registry.list_entry_tags(id).unwrap().contains(&&tag));
        assert!(registry.list_entry_tags(id).unwrap().contains(&&second));
        assert_eq!(registry.untag_entry(&tag, id), Some(id));
        assert_eq!(registry.list_entry_tags(id), Some(vec![&second]));
        assert_eq!(registry.untag_entry(&tag, id), None);
        assert!(registry.untag_entry(&second, id).is_some());
        assert_eq!(registry.list_entry_tags(id), None);
    }

    #[test]
    fn adds_multiple_entries() {
        let path = PathBuf::from("/tmp");
        let entry = EntryData { path };
        let mut registry = TagRegistry::default();
        registry.add_or_update_entry(entry);
        let path = PathBuf::from("/tmp/test");
        let entry = EntryData { path };
        registry.add_or_update_entry(entry);

        assert_eq!(registry.list_entries().count(), 2);
    }

    #[test]
    fn updates_tag_color() {
        let path = PathBuf::from("/tmp");
        let entry = EntryData { path: path.clone() };

        let mut registry = TagRegistry::default();
        registry.add_or_update_entry(entry);

        let id = registry.find_entry(&path).unwrap();

        let tag = Tag::new("test", Black);

        assert!(registry.tag_entry(&tag, id).is_none());
        assert!(registry.update_tag_color("test", Red));
        assert_eq!(registry.list_tags().next().unwrap().color(), &Red);
    }
}
