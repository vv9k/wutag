#![allow(dead_code)]
use crate::Result;
use wutag_core::color::Color;
use wutag_core::glob::Glob;
use wutag_core::registry::EntryData;
use wutag_core::tag::Tag;
use wutag_ipc::{IpcClient, Request, Response};

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ClientError {
    #[error("failed to tag files - {0}")]
    TagFiles(String),
    #[error("failed to untag files - {0}")]
    UntagFiles(String),
    #[error("failed to edit tag - {0}")]
    EditTag(String),
    #[error("failed to copy tags - {0}")]
    CopyTags(String),
    #[error("failed to clear files - {0}")]
    ClearFiles(String),
    #[error("failed to clear tags - {0}")]
    ClearTags(String),
    #[error("failed to list files - {0}")]
    ListFiles(String),
    #[error("failed to list tags - {0}")]
    ListTags(String),
    #[error("failed to inspect files - {0}")]
    InspectFiles(String),
    #[error("failed to search - {0}")]
    Search(String),
    #[error("failed to ping - {0}")]
    Ping(String),
    #[error("failed to clear cache - {0}")]
    ClearCache(String),
    #[error("unexpected response {0:?}")]
    UnexpectedResponse(HandledResponse),
}

#[derive(Debug)]
pub enum HandledResponse {
    TagFiles,
    UntagFiles,
    EditTag,
    CopyTags,
    ClearFiles,
    ClearTags,
    ListTags(HashMap<Tag, Vec<EntryData>>),
    ListFiles(Vec<(EntryData, Vec<Tag>)>),
    InspectFiles(Vec<(EntryData, Vec<Tag>)>),
    Search(Vec<EntryData>),
    Ping,
    ClearCache,
}

pub struct Client {
    client: IpcClient,
}

fn map_response(response: Response) -> Result<HandledResponse> {
    fn format_multiple_errors(e: Vec<String>) -> String {
        const SEPARATOR: &str = "\n - ";
        format!("{SEPARATOR}{}", e.join(SEPARATOR))
    }
    match response {
        Response::TagFiles(inner) => inner
            .to_result(|e| ClientError::TagFiles(format_multiple_errors(e)).into())
            .map(|_| HandledResponse::TagFiles),
        Response::UntagFiles(inner) => inner
            .to_result(|e| ClientError::UntagFiles(format_multiple_errors(e)).into())
            .map(|_| HandledResponse::UntagFiles),
        Response::EditTag(inner) => inner
            .to_result(|e| ClientError::EditTag(e).into())
            .map(|_| HandledResponse::EditTag),
        Response::CopyTags(inner) => inner
            .to_result(|e| ClientError::CopyTags(format_multiple_errors(e)).into())
            .map(|_| HandledResponse::CopyTags),
        Response::ClearFiles(inner) => inner
            .to_result(|e| ClientError::ClearFiles(format_multiple_errors(e)).into())
            .map(|_| HandledResponse::ClearFiles),
        Response::ClearTags(inner) => inner
            .to_result(|e| ClientError::ClearTags(format_multiple_errors(e)).into())
            .map(|_| HandledResponse::ClearTags),
        Response::ListFiles(inner) => inner
            .to_result(|e| ClientError::ListFiles(e).into())
            .map(HandledResponse::ListFiles),
        Response::ListTags(inner) => inner
            .to_result(|e| ClientError::ListTags(e).into())
            .map(HandledResponse::ListTags),
        Response::InspectFiles(inner) => inner
            .to_result(|e| ClientError::InspectFiles(e).into())
            .map(HandledResponse::InspectFiles),
        Response::Search(inner) => inner
            .to_result(|e| ClientError::Search(e).into())
            .map(HandledResponse::Search),
        Response::Ping(inner) => inner
            .to_result(|e| ClientError::Ping(e).into())
            .map(|_| HandledResponse::Ping),
        Response::ClearCache(inner) => inner
            .to_result(|e| ClientError::ClearCache(e).into())
            .map(|_| HandledResponse::ClearCache),
    }
}

impl Client {
    pub fn new(socket: impl Into<String>) -> Self {
        Self {
            client: IpcClient::new(socket),
        }
    }

    fn tag_files_impl(&self, request: Request) -> Result<()> {
        debug_assert!(matches!(
            request,
            Request::TagFiles { .. } | Request::TagFilesPattern { .. }
        ));
        self.client
            .request(request)
            .map_err(|e| ClientError::TagFiles(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    pub fn tag_files<P: AsRef<Path>>(
        &self,
        files: impl IntoIterator<Item = P>,
        tags: impl IntoIterator<Item = Tag>,
    ) -> Result<()> {
        self.tag_files_impl(Request::TagFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
            tags: tags.into_iter().collect(),
        })
    }

    pub fn tag_files_pattern(&self, glob: Glob, tags: impl IntoIterator<Item = Tag>) -> Result<()> {
        self.tag_files_impl(Request::TagFilesPattern {
            glob,

            tags: tags.into_iter().collect(),
        })
    }

    fn untag_files_impl(&self, request: Request) -> Result<()> {
        debug_assert!(matches!(
            request,
            Request::UntagFiles { .. } | Request::UntagFilesPattern { .. }
        ));
        self.client
            .request(request)
            .map_err(|e| ClientError::UntagFiles(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    pub fn untag_files<P: AsRef<Path>>(
        &self,
        files: impl IntoIterator<Item = P>,
        tags: impl IntoIterator<Item = Tag>,
    ) -> Result<()> {
        self.untag_files_impl(Request::UntagFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
            tags: tags.into_iter().collect(),
        })
    }

    pub fn untag_files_pattern(
        &self,
        glob: Glob,
        tags: impl IntoIterator<Item = Tag>,
    ) -> Result<()> {
        self.untag_files_impl(Request::UntagFilesPattern {
            glob,
            tags: tags.into_iter().collect(),
        })
    }

    pub fn edit_tag(&self, tag: String, color: Color) -> Result<()> {
        self.client
            .request(Request::EditTag { tag, color })
            .map_err(|e| ClientError::EditTag(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    fn copy_tags_impl(&self, request: Request) -> Result<()> {
        debug_assert!(matches!(
            request,
            Request::CopyTags { .. } | Request::CopyTagsPattern { .. }
        ));
        self.client
            .request(request)
            .map_err(|e| ClientError::CopyTags(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    pub fn copy_tags<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        target: impl IntoIterator<Item = P2>,
    ) -> Result<()> {
        self.client
            .request(Request::CopyTags {
                source: source.as_ref().to_path_buf(),
                target: target
                    .into_iter()
                    .map(|p| p.as_ref().to_path_buf())
                    .collect(),
            })
            .map_err(|e| ClientError::CopyTags(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    pub fn copy_tags_pattern(&self, source: impl AsRef<Path>, glob: Glob) -> Result<()> {
        self.copy_tags_impl(Request::CopyTagsPattern {
            glob,
            source: source.as_ref().to_path_buf(),
        })
    }

    fn clear_files_impl(&self, request: Request) -> Result<()> {
        debug_assert!(matches!(
            request,
            Request::ClearFiles { .. } | Request::ClearFilesPattern { .. }
        ));
        self.client
            .request(request)
            .map_err(|e| ClientError::ClearFiles(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    pub fn clear_files<P: AsRef<Path>>(&self, files: impl IntoIterator<Item = P>) -> Result<()> {
        self.clear_files_impl(Request::ClearFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
        })
    }

    pub fn clear_files_pattern(&self, glob: Glob) -> Result<()> {
        self.clear_files_impl(Request::ClearFilesPattern { glob })
    }

    pub fn clear_tags<T: AsRef<str>>(&self, tags: impl IntoIterator<Item = T>) -> Result<()> {
        self.client
            .request(Request::ClearTags {
                tags: tags.into_iter().map(|t| t.as_ref().to_string()).collect(),
            })
            .map_err(|e| ClientError::ClearTags(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    pub fn list_tags(&self, with_files: bool) -> Result<HashMap<Tag, Vec<EntryData>>> {
        self.client
            .request(Request::ListTags { with_files })
            .map_err(|e| ClientError::ListTags(e.to_string()).into())
            .and_then(map_response)
            .and_then(|r| {
                if let HandledResponse::ListTags(tags) = r {
                    Ok(tags)
                } else {
                    Err(ClientError::UnexpectedResponse(r).into())
                }
            })
    }

    pub fn list_files(&self, with_tags: bool) -> Result<Vec<(EntryData, Vec<Tag>)>> {
        self.client
            .request(Request::ListFiles { with_tags })
            .map_err(|e| ClientError::ListFiles(e.to_string()).into())
            .and_then(map_response)
            .and_then(|r| {
                if let HandledResponse::ListFiles(files) = r {
                    Ok(files)
                } else {
                    Err(ClientError::UnexpectedResponse(r).into())
                }
            })
    }

    fn inspect_files_impl(&self, request: Request) -> Result<Vec<(EntryData, Vec<Tag>)>> {
        debug_assert!(matches!(
            request,
            Request::InspectFiles { files: _ } | Request::InspectFilesPattern { .. }
        ));
        self.client
            .request(request)
            .map_err(|e| ClientError::InspectFiles(e.to_string()).into())
            .and_then(map_response)
            .and_then(|r| {
                if let HandledResponse::InspectFiles(files) = r {
                    Ok(files)
                } else {
                    Err(ClientError::UnexpectedResponse(r).into())
                }
            })
    }

    pub fn inspect_files<P: AsRef<Path>>(
        &self,
        files: impl IntoIterator<Item = P>,
    ) -> Result<Vec<(EntryData, Vec<Tag>)>> {
        self.inspect_files_impl(Request::InspectFiles {
            files: files
                .into_iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect(),
        })
    }

    pub fn inspect_files_pattern(&self, glob: Glob) -> Result<Vec<(EntryData, Vec<Tag>)>> {
        self.inspect_files_impl(Request::InspectFilesPattern { glob })
    }

    pub fn search<S: Into<String>>(
        &self,
        tags: impl IntoIterator<Item = S>,
        any: bool,
    ) -> Result<Vec<EntryData>> {
        self.client
            .request(Request::Search {
                tags: tags.into_iter().map(S::into).collect(),
                any,
            })
            .map_err(|e| ClientError::Search(e.to_string()).into())
            .and_then(map_response)
            .and_then(|r| {
                if let HandledResponse::Search(files) = r {
                    Ok(files)
                } else {
                    Err(ClientError::UnexpectedResponse(r).into())
                }
            })
    }

    pub fn ping(&self) -> Result<()> {
        self.client
            .request(Request::Ping)
            .map_err(|e| ClientError::Ping(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }

    pub fn clear_cache(&self) -> Result<()> {
        self.client
            .request(Request::ClearCache)
            .map_err(|e| ClientError::ClearCache(e.to_string()).into())
            .and_then(map_response)
            .map(|_| ())
    }
}
