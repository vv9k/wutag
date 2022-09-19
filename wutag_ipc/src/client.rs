use crate::{IpcError, Request, Response, Result, REQUEST_SEPARATOR};
use interprocess::local_socket::LocalSocketStream;
use std::io::{self, prelude::*, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to initialize connection - {0}")]
    ConnectionInit(io::Error),
    #[error("failed to read from socket - {0}")]
    ConnectionRead(io::Error),
    #[error("failed to write to socket - {0}")]
    ConnectionWrite(io::Error),
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

    pub fn request(&self, request: Request) -> Result<Response> {
        let conn =
            LocalSocketStream::connect(self.path.as_str()).map_err(ClientError::ConnectionInit)?;
        let mut conn = BufReader::new(conn);

        self.send_request(request, &mut conn)?;
        let response = self.read_response(&mut conn)?;

        Ok(response)
    }

    fn send_request(
        &self,
        request: Request,
        conn: &mut BufReader<LocalSocketStream>,
    ) -> Result<()> {
        let payload = request.to_payload()?;
        conn.get_mut()
            .write_all(&payload)
            .map_err(ClientError::ConnectionWrite)
            .map_err(IpcError::Client)
            .map(|_| ())
    }

    fn read_response(&self, conn: &mut BufReader<LocalSocketStream>) -> Result<Response> {
        let mut buf = vec![];
        conn.read_until(REQUEST_SEPARATOR, &mut buf)
            .map_err(ClientError::ConnectionRead)?;

        Response::from_payload(&buf)
    }
}
