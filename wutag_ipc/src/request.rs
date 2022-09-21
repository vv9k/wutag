use crate::{IpcError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use wutag_core::color::Color;
use wutag_core::glob::Glob;
use wutag_core::registry::EntryData;
use wutag_core::tag::Tag;

#[derive(Debug, Error)]
pub enum PayloadError {
    #[error("Failed to serialize as cbor - {0}")]
    Serialize(serde_cbor::Error),
    #[error("Failed to deserialize cbor payload - {0}")]
    Deserialize(serde_cbor::Error),
}

#[derive(Deserialize, Debug, Serialize)]
pub enum Request {
    TagFiles {
        files: Vec<PathBuf>,
        tags: Vec<Tag>,
    },
    TagFilesPattern {
        glob: Glob,
        tags: Vec<Tag>,
    },
    UntagFiles {
        files: Vec<PathBuf>,
        tags: Vec<Tag>,
    },
    UntagFilesPattern {
        glob: Glob,
        tags: Vec<Tag>,
    },
    EditTag {
        tag: String,
        color: Color,
    },
    ClearFiles {
        files: Vec<PathBuf>,
    },
    ClearFilesPattern {
        glob: Glob,
    },
    ClearTags {
        tags: Vec<String>,
    },
    CopyTags {
        source: PathBuf,
        target: Vec<PathBuf>,
    },
    CopyTagsPattern {
        source: PathBuf,
        glob: Glob,
    },
    ListTags {
        with_files: bool,
    },
    ListFiles {
        with_tags: bool,
    },
    InspectFiles {
        files: Vec<PathBuf>,
    },
    InspectFilesPattern {
        glob: Glob,
    },
    Search {
        tags: Vec<String>,
        any: bool,
    },
    Ping,
    ClearCache,
}

impl Request {
    pub fn to_payload(&self) -> Result<Vec<u8>> {
        to_payload(self)
    }

    pub fn from_payload(bytes: &[u8]) -> Result<Self> {
        from_payload(bytes)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub enum RequestResult<T, E> {
    Ok(T),
    Error(E),
}

impl<T, E> RequestResult<T, E> {
    /// Converts this request result to std::result::Result by applying the `make_error_fn` to
    /// the inner error
    pub fn to_result<E2: std::error::Error>(
        self,
        make_error_fn: impl FnOnce(E) -> E2,
    ) -> std::result::Result<T, E2> {
        match self {
            RequestResult::Ok(ok) => Ok(ok),
            RequestResult::Error(e) => Err(make_error_fn(e)),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub enum Response {
    TagFiles(RequestResult<(), Vec<String>>),
    UntagFiles(RequestResult<(), Vec<String>>),
    EditTag(RequestResult<(), String>),
    CopyTags(RequestResult<(), Vec<String>>),
    ClearFiles(RequestResult<(), Vec<String>>),
    ClearTags(RequestResult<(), Vec<String>>),
    ListTags(RequestResult<HashMap<Tag, Vec<EntryData>>, String>),
    #[allow(clippy::type_complexity)]
    ListFiles(RequestResult<Vec<(EntryData, Vec<Tag>)>, String>),
    InspectFiles(RequestResult<Vec<(EntryData, Vec<Tag>)>, String>),
    Search(RequestResult<Vec<EntryData>, String>),
    Ping(RequestResult<(), String>),
    ClearCache(RequestResult<(), String>),
}

impl Response {
    pub fn to_payload(&self) -> Result<Vec<u8>> {
        to_payload(self)
    }
    pub fn from_payload(bytes: &[u8]) -> Result<Self> {
        from_payload(bytes)
    }
}

fn to_payload<T: Serialize>(item: &T) -> Result<Vec<u8>> {
    serde_cbor::to_vec(item)
        .map_err(PayloadError::Serialize)
        .map_err(IpcError::Payload)
}

fn from_payload<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T> {
    serde_cbor::from_slice(bytes)
        .map_err(PayloadError::Deserialize)
        .map_err(IpcError::Payload)
}
