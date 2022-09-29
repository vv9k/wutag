use crate::{IpcError, Result};
use interprocess::local_socket::LocalSocketStream;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{prelude::*, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayloadError {
    #[error("Failed to serialize as cbor - {0}")]
    Serialize(serde_cbor::Error),
    #[error("Failed to deserialize cbor payload - {0}")]
    Deserialize(serde_cbor::Error),
}

#[derive(Deserialize, Debug, Serialize)]
pub enum PayloadResult<T, E> {
    Ok(T),
    Error(E),
}

impl<T, E> PayloadResult<T, E> {
    /// Converts this request result to std::result::Result by applying the `make_error_fn` to
    /// the inner error
    pub fn to_result<E2: std::error::Error>(
        self,
        make_error_fn: impl FnOnce(E) -> E2,
    ) -> std::result::Result<T, E2> {
        match self {
            PayloadResult::Ok(ok) => Ok(ok),
            PayloadResult::Error(e) => Err(make_error_fn(e)),
        }
    }
}

pub trait Payload: Sized + DeserializeOwned + Serialize + std::fmt::Debug {
    fn to_payload(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self)
            .map_err(PayloadError::Serialize)
            .map_err(IpcError::Payload)
    }

    fn from_payload(bytes: &[u8]) -> Result<Self> {
        serde_cbor::from_slice(bytes)
            .map_err(PayloadError::Deserialize)
            .map_err(IpcError::Payload)
    }

    fn send(&self, conn: &mut BufReader<LocalSocketStream>) -> Result<()> {
        let payload = self.to_payload()?;
        send_payload(&payload, conn)
    }

    fn read(conn: &mut BufReader<LocalSocketStream>) -> Result<Self> {
        let payload = read_payload(conn)?;
        Self::from_payload(&payload)
    }
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
