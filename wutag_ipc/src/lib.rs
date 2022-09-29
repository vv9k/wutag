mod client;
mod payload;
mod server;

pub use client::{ClientError, IpcClient};
pub use payload::{Payload, PayloadError, PayloadResult};
pub use server::{IpcServer, ServerError};

use interprocess::local_socket::NameTypeSupport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use wutag_core::color::Color;
use wutag_core::glob::Glob;
use wutag_core::registry::EntryData;
use wutag_core::tag::Tag;

pub type Result<T> = std::result::Result<T, IpcError>;

pub fn socket_name(base_path: impl AsRef<Path>, name: impl AsRef<str>) -> String {
    use NameTypeSupport::*;
    let name = name.as_ref();
    match NameTypeSupport::query() {
        OnlyPaths => base_path.as_ref().join(name).to_string_lossy().to_string(),
        OnlyNamespaced | Both => format!("@{name}"),
    }
}

pub fn default_socket() -> String {
    let username = whoami::username();
    let socketname = format!("wutag-{username}.sock");
    let dir = dirs::runtime_dir()
        .or_else(dirs::data_local_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp".into());
    socket_name(dir, socketname)
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("{0}")]
    Payload(#[from] PayloadError),
    #[error("{0}")]
    Server(#[from] ServerError),
    #[error("{0}")]
    Client(#[from] ClientError),
    #[error("failed to read from socket - {0}")]
    ConnectionRead(io::Error),
    #[error("failed to write to socket - {0}")]
    ConnectionWrite(io::Error),
    #[error("Error: {0}")]
    Other(String),
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

impl Payload for Request {}

#[derive(Deserialize, Debug, Serialize)]
pub enum Response {
    TagFiles(PayloadResult<(), Vec<String>>),
    UntagFiles(PayloadResult<(), Vec<String>>),
    EditTag(PayloadResult<(), String>),
    CopyTags(PayloadResult<(), Vec<String>>),
    ClearFiles(PayloadResult<(), Vec<String>>),
    ClearTags(PayloadResult<(), Vec<String>>),
    ListTags(PayloadResult<HashMap<Tag, Vec<EntryData>>, String>),
    ListFiles(PayloadResult<Vec<(EntryData, Vec<Tag>)>, String>),
    InspectFiles(PayloadResult<Vec<(EntryData, Vec<Tag>)>, String>),
    Search(PayloadResult<Vec<EntryData>, String>),
    Ping(PayloadResult<(), String>),
    ClearCache(PayloadResult<(), String>),
}

impl Payload for Response {}
