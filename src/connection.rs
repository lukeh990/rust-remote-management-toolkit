/*
connection.rs
This module provides a high level interface for interacting with the RRMT protocol.

To-Do:
- [ ] Handle
 */

use std::net::SocketAddr;
use std::ops::MulAssign;
use std::time::Duration;
use std::{error, fmt};

use tokio::net::TcpStream;
use tokio::time::sleep;
use uuid::Uuid;

use crate::frame::{read_frame, write_frame, RRMTFrame};

#[derive(Debug, Clone)]
pub enum AuthorizationError {
    AlreadyAuthorized,
    InvalidToken,
    TransmissionError(String),
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            AuthorizationError::AlreadyAuthorized => "Already Authorized",
            AuthorizationError::InvalidToken => "Token Invalid",
            AuthorizationError::TransmissionError(msg) => msg,
        };
        write!(f, "{}", message)
    }
}

impl error::Error for AuthorizationError {}

pub struct Connection {
    stream: TcpStream,
    authorized: bool,
    pub machine_id: Uuid,
}

impl Connection {
    pub async fn new(addr: SocketAddr, machine_id: Uuid) -> crate::Result<Connection> {
        let stream = establish_stream_backoff(addr).await?;
        Ok(Connection {
            stream,
            authorized: false,
            machine_id,
        })
    }

    pub async fn authorize(&mut self) -> Result<(), AuthorizationError> {
        if self.authorized {
            return Err(AuthorizationError::AlreadyAuthorized);
        }

        if write_frame(&mut self.stream, RRMTFrame::Authorize(self.machine_id))
            .await
            .is_err()
        {
            return Err(AuthorizationError::TransmissionError(
                "Send Failure".to_string(),
            ));
        }

        match read_frame(&mut self.stream).await {
            Ok(frame) => match frame {
                RRMTFrame::Accepted => {
                    self.authorized = true;
                    Ok(())
                }

                RRMTFrame::Denied => Err(AuthorizationError::InvalidToken),

                RRMTFrame::Error(error_type, msg) => Err(AuthorizationError::TransmissionError(
                    format!("{:?} | {}", error_type, msg),
                )),

                _ => Err(AuthorizationError::TransmissionError(
                    "Invalid Response".to_string(),
                )),
            },
            Err(error) => Err(AuthorizationError::TransmissionError(error.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
struct EstablishStreamError;

impl fmt::Display for EstablishStreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to establish stream")
    }
}

impl error::Error for EstablishStreamError {}

async fn establish_stream_backoff(addr: SocketAddr) -> Result<TcpStream, EstablishStreamError> {
    let mut backoff = Duration::from_millis(1000);
    let mut i = 0;
    loop {
        sleep(backoff).await;
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                return Ok(stream);
            }
            Err(_) => {
                if i >= 5 {
                    return Err(EstablishStreamError);
                }
                backoff.mul_assign(2);
                println!(
                    "Failed to connect retrying after {} seconds",
                    backoff.as_secs()
                );
                i += 1;
                continue;
            }
        };
    }
}
