use crate::{IpcError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use wutag_core::color::Color;
use wutag_core::registry::EntryData;
use wutag_core::tag::Tag;

pub const REQUEST_SEPARATOR: u8 = 4; // EOT

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
    UntagFiles {
        files: Vec<PathBuf>,
        tags: Vec<Tag>,
    },
    EditTag {
        tag: String,
        color: Color,
    },
    ClearTags {
        files: Vec<PathBuf>,
    },
    CopyTags {
        source: PathBuf,
        target: Vec<PathBuf>,
    },
    ListTags,
    ListFiles {
        with_tags: bool,
    },
    InspectFiles {
        files: Vec<PathBuf>,
    },
    Search {
        tags: Vec<String>,
        any: bool,
    },
    Ping,
    CleanCache,
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

#[derive(Deserialize, Debug, Serialize)]
pub enum Response {
    TagFiles(RequestResult<(), Vec<String>>),
    UntagFiles(RequestResult<(), Vec<String>>),
    EditTag(RequestResult<(), String>),
    CopyTags(RequestResult<(), Vec<String>>),
    ClearTags(RequestResult<(), Vec<String>>),
    ListTags(RequestResult<Vec<Tag>, String>),
    #[allow(clippy::type_complexity)]
    ListFiles(RequestResult<Vec<(EntryData, Option<Vec<Tag>>)>, String>),
    InspectFiles(RequestResult<Vec<(EntryData, Vec<Tag>)>, String>),
    Search(RequestResult<Vec<EntryData>, String>),
    Ping(RequestResult<(), String>),
    CleanCache(RequestResult<(), String>),
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
        .map(|mut payload| {
            payload.push(REQUEST_SEPARATOR);
            payload
        })
        .map_err(PayloadError::Serialize)
        .map_err(IpcError::Payload)
}

fn from_payload<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T> {
    serde_cbor::from_slice(&bytes[..bytes.len() - 1])
        .map_err(PayloadError::Deserialize)
        .map_err(IpcError::Payload)
}
