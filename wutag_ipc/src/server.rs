use crate::{IpcError, Request, Response, Result, REQUEST_SEPARATOR};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::collections::VecDeque;
use std::io::{self, prelude::*, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("failed to accept connection - {0}")]
    ConnectionAccept(io::Error),
    #[error("failed to read from socket - {0}")]
    ConnectionRead(io::Error),
    #[error("failed to write to socket - {0}")]
    ConnectionWrite(io::Error),
    #[error("failed to send response - no active connection")]
    NoActiveConnection,
    #[error("failed to bind local listener - {0}")]
    Bind(io::Error),
}

pub struct IpcServer {
    #[allow(dead_code)]
    path: String,
    socket: LocalSocketListener,
    conns: VecDeque<BufReader<LocalSocketStream>>,
}

impl IpcServer {
    pub fn new(path: impl Into<String>) -> Result<Self> {
        let path = path.into();
        let socket = LocalSocketListener::bind(path.as_str()).map_err(ServerError::Bind)?;
        Ok(Self {
            path,
            socket,
            conns: VecDeque::new(),
        })
    }

    pub fn accept_request(&mut self) -> Result<Request> {
        let mut buf = Vec::with_capacity(128);

        let conn = self
            .socket
            .accept()
            .map_err(ServerError::ConnectionAccept)?;
        let mut conn = BufReader::new(conn);
        conn.read_until(REQUEST_SEPARATOR, &mut buf)
            .map_err(ServerError::ConnectionRead)?;

        let request = Request::from_payload(&buf)?;
        log::trace!("{request:?}");
        self.conns.push_back(conn);
        Ok(request)
    }

    pub fn send_response(&mut self, response: Response) -> Result<()> {
        if let Some(mut conn) = self.conns.pop_front() {
            let payload = response.to_payload()?;

            return conn
                .get_mut()
                .write_all(&payload)
                .map_err(ServerError::ConnectionWrite)
                .map_err(IpcError::Server);
        }

        Err(ServerError::NoActiveConnection).map_err(IpcError::Server)
    }
}
