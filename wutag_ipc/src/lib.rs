mod client;
mod request;
mod server;

pub use client::{ClientError, IpcClient};
pub use request::{PayloadError, Request, RequestResult, Response};
pub use server::{IpcServer, ServerError};

use interprocess::local_socket::{LocalSocketStream, NameTypeSupport};
use std::io::{self, prelude::*, BufReader};
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

fn send_payload(payload: &[u8], conn: &mut BufReader<LocalSocketStream>) -> Result<()> {
    let mut size = payload.len().to_be_bytes().to_vec();
    let conn = conn.get_mut();
    size.extend(payload);
    conn.write_all(&size)
        .map_err(IpcError::ConnectionWrite)
        .map(|_| ())
}

fn read_payload(conn: &mut BufReader<LocalSocketStream>) -> Result<Vec<u8>> {
    let mut size = [0u8; 8];
    conn.read_exact(&mut size)
        .map_err(IpcError::ConnectionRead)?;
    let size = u64::from_be_bytes(size);

    let mut buf = vec![0; size as usize];
    conn.read_exact(&mut buf)
        .map_err(IpcError::ConnectionRead)
        .map(|_| buf)
}
