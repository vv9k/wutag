use crate::{payload::Payload, IpcError, Result};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::collections::VecDeque;
use std::io::{self, BufReader};
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

    pub fn accept_request<REQUEST: Payload>(&mut self) -> Result<REQUEST> {
        let conn = self
            .socket
            .accept()
            .map_err(ServerError::ConnectionAccept)?;
        let mut conn = BufReader::new(conn);
        let request = REQUEST::read(&mut conn)?;
        log::debug!("got request: {request:?}");
        self.conns.push_back(conn);
        Ok(request)
    }

    pub fn send_response<RESPONSE: Payload>(&mut self, response: RESPONSE) -> Result<()> {
        if let Some(mut conn) = self.conns.pop_front() {
            log::debug!("sending response: {response:?}");
            return response.send(&mut conn);
        }

        Err(ServerError::NoActiveConnection).map_err(IpcError::Server)
    }
}
