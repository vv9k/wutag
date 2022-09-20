mod client;
mod request;
mod server;

pub use client::{ClientError, IpcClient};
pub use request::{PayloadError, Request, RequestResult, Response};
pub use server::{IpcServer, ServerError};

use interprocess::local_socket::NameTypeSupport;
use std::path::Path;
use thiserror::Error;

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
    socket_name("/tmp", "wutag.sock")
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("{0}")]
    Payload(#[from] PayloadError),
    #[error("{0}")]
    Server(#[from] ServerError),
    #[error("{0}")]
    Client(#[from] ClientError),
    #[error("Error: {0}")]
    Other(String),
}
