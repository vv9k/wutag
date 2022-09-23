use crate::{read_payload, send_payload, IpcError, Request, Response, Result};
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

    pub fn accept_request(&mut self) -> Result<Request> {
        let conn = self
            .socket
            .accept()
            .map_err(ServerError::ConnectionAccept)?;
        let mut conn = BufReader::new(conn);
        let request = read_payload(&mut conn).and_then(|buf| Request::from_payload(&buf))?;
        log::debug!("got request: {request:?}");
        self.conns.push_back(conn);
        Ok(request)
    }

    pub fn send_response(&mut self, response: Response) -> Result<()> {
        if let Some(mut conn) = self.conns.pop_front() {
            log::debug!("sending response: {response:?}");
            let payload = response.to_payload()?;
            return send_payload(&payload, &mut conn);
        }

        Err(ServerError::NoActiveConnection).map_err(IpcError::Server)
    }
}
