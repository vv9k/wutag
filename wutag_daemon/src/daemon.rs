use crate::registry::{get_registry_read, get_registry_write};
use crate::{EntryEvent, ENTRIES_EVENTS};
use anyhow::{Context, Result};
use std::path::PathBuf;
use wutag_core::color::{Color, DEFAULT_COLORS};
use wutag_core::registry::EntryData;
use wutag_core::tag::{clear_tags, list_tags, Tag};
use wutag_ipc::{IpcServer, Request, RequestResult, Response};

pub struct WutagDaemon {
    listener: IpcServer,
}

impl WutagDaemon {
    pub fn new(listener: IpcServer) -> Result<Self> {
        Ok(Self { listener })
    }

    pub fn process_connection(&mut self) -> Result<()> {
        let request = self
            .listener
            .accept_request()
            .context("failed to accept request")?;
        let response = self.process_request(request);
        log::trace!("{response:?}");
        self.listener
            .send_response(response)
            .context("failed to send response")
    }

    fn process_request(&mut self, request: Request) -> Response {
        match request {
            Request::TagFiles { files, tags } => self.tag_files(files, tags),
            Request::UntagFiles { files, tags } => self.untag_files(files, tags),
            Request::ListTags => self.list_tags(),
            Request::ListFiles { with_tags } => self.list_files(with_tags),
            Request::InspectFiles { files } => self.inspect_files(files),
            Request::ClearFiles { files } => self.clear_files(files),
            Request::ClearTags { tags } => self.clear_tags(tags),
            Request::Search { tags, any } => self.search(tags, any),
            Request::CopyTags { source, target } => self.copy_tags(source, target),
            Request::Ping => self.ping(),
            Request::EditTag { tag, color } => self.edit_tag(tag, color),
            Request::CleanCache => self.clean_cache(),
        }
    }

    fn tag_files(&mut self, files: Vec<PathBuf>, tags: Vec<Tag>) -> Response {
        if files.is_empty() {
            return Response::TagFiles(RequestResult::Error(vec!["no files to tag".into()]));
        }
        if tags.is_empty() {
            return Response::TagFiles(RequestResult::Error(vec!["no tags provided".into()]));
        }
        let mut errors = vec![];
        let mut new_entries = vec![];
        let mut registry = get_registry_write();

        for file in &files {
            log::trace!("processing file {}", file.display());
            let entry = EntryData::new(file);
            let (id, added) = registry.add_or_update_entry(entry);
            if added {
                if let Err(e) = clear_tags(file) {
                    log::error!(
                        "failed to clear tags of file `{}`, reason: {e}",
                        file.display()
                    );
                }
                new_entries.push(file.to_path_buf());
            }
            for tag in &tags {
                log::trace!("tagging file {}, tag {tag}", file.display());
                if let Err(e) = tag.save_to(file) {
                    errors.push(format!(
                        "Error for `{}` tag: `{tag}`, reason: {e}",
                        file.display()
                    ));
                } else {
                    registry.tag_entry(tag, id);
                }
            }
            if registry.list_entry_tags(id).unwrap_or_default().is_empty() {
                registry.remove_entry(id);
            }
        }

        if let Err(e) = registry.save() {
            log::error!("{e}")
        }

        if !new_entries.is_empty() {
            match ENTRIES_EVENTS.try_write() {
                Ok(mut events) => {
                    events.push_back(EntryEvent::Add(new_entries));
                }
                Err(e) => {
                    log::error!("failed to lock entries events, reason: {e}");
                }
            }
        }

        if errors.is_empty() {
            Response::TagFiles(RequestResult::Ok(()))
        } else {
            Response::TagFiles(RequestResult::Error(errors))
        }
    }

    fn untag_files(&mut self, files: Vec<PathBuf>, tags: Vec<Tag>) -> Response {
        if files.is_empty() {
            return Response::UntagFiles(RequestResult::Error(vec!["no files to untag".into()]));
        }
        if tags.is_empty() {
            return Response::UntagFiles(RequestResult::Error(vec!["no tags provided".into()]));
        }
        let mut registry = get_registry_write();
        let mut errors = vec![];
        let mut removed = vec![];

        for file in &files {
            if let Some(id) = registry.find_entry(file) {
                for tag in &tags {
                    if let Err(e) = tag.remove_from(file) {
                        errors.push(format!("{} tag: {tag}, error: {e}", file.display()));
                    } else if let Some(entry) = registry.untag_entry(tag, id) {
                        removed.push(entry.into_path_buf());
                    }
                }
            }
        }

        if let Err(e) = registry.save() {
            log::error!("{e}")
        }

        if !removed.is_empty() {
            match ENTRIES_EVENTS.try_write() {
                Ok(mut events) => {
                    events.push_back(EntryEvent::Remove(removed));
                }
                Err(e) => {
                    log::error!("failed to lock entries events, reason: {e}");
                }
            }
        }

        if errors.is_empty() {
            Response::UntagFiles(RequestResult::Ok(()))
        } else {
            Response::UntagFiles(RequestResult::Error(errors))
        }
    }

    fn edit_tag(&mut self, tag: String, color: Color) -> Response {
        let mut registry = get_registry_write();
        if registry.get_tag(&tag).is_none() {
            return Response::EditTag(RequestResult::Error(format!("tag {tag} doesn't exist")));
        }
        registry.update_tag_color(tag, color);
        if let Err(e) = registry.save() {
            log::error!("{e}")
        }
        Response::EditTag(RequestResult::Ok(()))
    }

    fn copy_tags(&mut self, source: PathBuf, target: Vec<PathBuf>) -> Response {
        let tags = match list_tags(&source) {
            Ok(tags) => tags,
            Err(e) => {
                return Response::CopyTags(RequestResult::Error(vec![format!(
                    "faile to copy tags - {e}"
                )]))
            }
        };
        if tags.is_empty() {
            return Response::CopyTags(RequestResult::Ok(()));
        }

        let mut errors = vec![];
        let mut new_entries = vec![];
        let mut registry = get_registry_write();

        for path in target {
            let (id, added) = registry.add_or_update_entry(EntryData::new(&path));
            if added {
                if let Err(e) = clear_tags(&path) {
                    log::error!(
                        "failed to clear tags of file `{}`, reason: {e}",
                        path.display()
                    );
                }
                new_entries.push(path.to_path_buf());
            }
            for tag in &tags {
                if let Err(e) = tag.save_to(&path) {
                    errors.push(e.to_string());
                } else {
                    registry.tag_entry(tag, id);
                }
            }
            if registry.list_entry_tags(id).unwrap_or_default().is_empty() {
                registry.remove_entry(id);
            }
        }

        if let Err(e) = registry.save() {
            log::error!("{e}")
        }

        if !new_entries.is_empty() {
            match ENTRIES_EVENTS.try_write() {
                Ok(mut events) => {
                    events.push_back(EntryEvent::Add(new_entries));
                }
                Err(e) => {
                    log::error!("failed to lock entries events, reason: {e}");
                }
            }
        }

        if errors.is_empty() {
            Response::CopyTags(RequestResult::Ok(()))
        } else {
            Response::CopyTags(RequestResult::Error(errors))
        }
    }

    fn clear_files(&mut self, files: Vec<PathBuf>) -> Response {
        if files.is_empty() {
            return Response::ClearFiles(RequestResult::Error(vec!["no files to clear".into()]));
        }

        let mut errors = vec![];
        let mut registry = get_registry_write();

        for file in &files {
            if let Some(id) = registry.find_entry(file) {
                let entry = registry.get_entry(id).unwrap();
                if let Err(e) = clear_tags(entry.path()) {
                    errors.push(format!(
                        "failed to clear tags from `{}`, reason: {e}",
                        entry.path().display()
                    ));
                } else {
                    registry.clear_entry(id);
                }
            }
        }

        if let Err(e) = registry.save() {
            log::error!("{e}")
        }

        match ENTRIES_EVENTS.try_write() {
            Ok(mut events) => {
                events.push_back(EntryEvent::Remove(files));
            }
            Err(e) => {
                log::error!("failed to lock entries events, reason: {e}");
            }
        }

        if errors.is_empty() {
            Response::ClearFiles(RequestResult::Ok(()))
        } else {
            Response::ClearFiles(RequestResult::Error(errors))
        }
    }

    fn clear_tags(&mut self, tags: Vec<String>) -> Response {
        if tags.is_empty() {
            return Response::ClearTags(RequestResult::Error(vec!["no tags to clear".into()]));
        }

        let mut removed = vec![];
        let mut registry = get_registry_write();

        for tag in &tags {
            let tag = Tag::random(tag, DEFAULT_COLORS);
            let cleared = registry.clear_tag(&tag);
            if let Some(cleared) = cleared {
                cleared
                    .into_iter()
                    .map(|e| e.into_path_buf())
                    .for_each(|e| removed.push(e));
            }
        }

        if let Err(e) = registry.save() {
            log::error!("{e}")
        }

        match ENTRIES_EVENTS.try_write() {
            Ok(mut events) => {
                events.push_back(EntryEvent::Remove(removed));
            }
            Err(e) => {
                log::error!("failed to lock entries events, reason: {e}");
            }
        }

        Response::ClearFiles(RequestResult::Ok(()))
    }

    fn list_tags(&mut self) -> Response {
        Response::ListTags(RequestResult::Ok(
            get_registry_read().list_tags().cloned().collect(),
        ))
    }

    fn list_files(&mut self, with_tags: bool) -> Response {
        let registry = get_registry_read();
        let entries = if with_tags {
            registry.list_entries_and_tags().collect()
        } else {
            registry.list_entries().map(|e| (e.clone(), None)).collect()
        };
        Response::ListFiles(RequestResult::Ok(entries))
    }

    fn inspect_files(&mut self, files: Vec<PathBuf>) -> Response {
        if files.is_empty() {
            return Response::InspectFiles(RequestResult::Error("no files to inspect".into()));
        }
        let mut entries = vec![];

        let registry = get_registry_read();
        for file in files {
            if let Some(id) = registry.find_entry(&file) {
                let tags = registry
                    .list_entry_tags(id)
                    .unwrap_or_default()
                    .into_iter()
                    .cloned()
                    .collect();
                let entry = registry.get_entry(id).unwrap().clone();
                entries.push((entry, tags));
            }
        }

        Response::InspectFiles(RequestResult::Ok(entries))
    }

    fn search(&mut self, tags: Vec<String>, any: bool) -> Response {
        if tags.is_empty() {
            return Response::Search(RequestResult::Error("no tags to search for".into()));
        }
        let registry = get_registry_read();
        let entries = if any {
            registry.list_entries_with_any_tags(tags)
        } else {
            registry.list_entries_with_all_tags(tags)
        };
        let mut found = vec![];
        for entry in entries {
            if let Some(entry) = registry.get_entry(entry) {
                found.push(entry.clone());
            }
        }
        Response::Search(RequestResult::Ok(found))
    }

    fn ping(&mut self) -> Response {
        Response::Ping(RequestResult::Ok(()))
    }

    fn clean_cache(&mut self) -> Response {
        let mut registry = get_registry_write();
        registry.clear();
        if let Err(e) = registry.save() {
            log::error!("{e}")
        }
        Response::CleanCache(RequestResult::Ok(()))
    }
}
