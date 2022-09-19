#![allow(dead_code)]
use wutag_core::color::Color;
use wutag_core::tag::Tag;
use wutag_ipc::{IpcClient, Request, Response, Result};

use std::path::Path;

pub struct Client {
    client: IpcClient,
}

impl Client {
    pub fn new(socket: impl Into<String>) -> Self {
        Self {
            client: IpcClient::new(socket),
        }
    }

    pub fn tag_files<P: AsRef<Path>>(
        &self,
        files: impl IntoIterator<Item = P>,
        tags: impl IntoIterator<Item = Tag>,
    ) -> Result<Response> {
        self.client.request(Request::TagFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
            tags: tags.into_iter().collect(),
        })
    }

    pub fn untag_files<P: AsRef<Path>>(
        &self,
        files: impl IntoIterator<Item = P>,
        tags: impl IntoIterator<Item = Tag>,
    ) -> Result<Response> {
        self.client.request(Request::UntagFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
            tags: tags.into_iter().collect(),
        })
    }

    pub fn edit_tag(&self, tag: String, color: Color) -> Result<Response> {
        self.client.request(Request::EditTag { tag, color })
    }

    pub fn copy_tags<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        target: impl IntoIterator<Item = P2>,
    ) -> Result<Response> {
        self.client.request(Request::CopyTags {
            source: source.as_ref().to_path_buf(),
            target: target
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
        })
    }

    pub fn clear_files<P: AsRef<Path>>(
        &self,
        files: impl IntoIterator<Item = P>,
    ) -> Result<Response> {
        self.client.request(Request::ClearFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
        })
    }

    pub fn clear_tags<T: AsRef<str>>(&self, tags: impl IntoIterator<Item = T>) -> Result<Response> {
        self.client.request(Request::ClearTags {
            tags: tags.into_iter().map(|t| t.as_ref().to_string()).collect(),
        })
    }

    pub fn list_tags(&self) -> Result<Response> {
        self.client.request(Request::ListTags)
    }

    pub fn list_files(&self, with_tags: bool) -> Result<Response> {
        self.client.request(Request::ListFiles { with_tags })
    }

    pub fn inspect_files<P: AsRef<Path>>(
        &self,
        files: impl IntoIterator<Item = P>,
    ) -> Result<Response> {
        self.client.request(Request::InspectFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
        })
    }

    pub fn search<S: Into<String>>(
        &self,
        tags: impl IntoIterator<Item = S>,
        any: bool,
    ) -> Result<Response> {
        self.client.request(Request::Search {
            tags: tags.into_iter().map(S::into).collect(),
            any,
        })
    }

    pub fn ping(&self) -> Result<()> {
        self.client.request(Request::Ping).map(|_| ())
    }

    pub fn clean_cache(&self) -> Result<()> {
        self.client.request(Request::CleanCache).map(|_| ())
    }
}
