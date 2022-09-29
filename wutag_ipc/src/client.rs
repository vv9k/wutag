use crate::{payload::Payload, Result};
use interprocess::local_socket::LocalSocketStream;
use std::io::{self, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to initialize connection - {0}")]
    ConnectionInit(io::Error),
    #[error("failed to send response - no active connection")]
    NoActiveConnection,
    #[error("failed to bind local listener - {0}")]
    Bind(io::Error),
}

pub struct IpcClient {
    path: String,
}

impl IpcClient {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn request<REQUEST: Payload, RESPONSE: Payload>(
        &self,
        request: REQUEST,
    ) -> Result<RESPONSE> {
        let conn =
            LocalSocketStream::connect(self.path.as_str()).map_err(ClientError::ConnectionInit)?;
        let mut conn = BufReader::new(conn);

        request.send(&mut conn)?;
        let response = RESPONSE::read(&mut conn)?;

        Ok(response)
    }
}
