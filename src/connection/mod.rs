/*
connection.rs
This module provides a high level interface for interacting with the RRMT protocol.

To-Do:
- [ ] Read/Write threads
- [ ] Connection API
- [ ] Authorize with server
- [ ] Respond to pings
- [ ] Pass around packets
- [ ] Handle network disconnects
 */

use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use uuid::Uuid;
use crate::frame::{ReadError, RRMTError, RRMTFrame, RRMTFrameType, RRMTInvalidRoleError};

pub mod error;

type Responder<T, E> = oneshot::Sender<Result<T, E>>;

pub struct WriteCommandResponder {
    oneshot: Responder<RRMTFrame, ReadError>,
    expected_types: Vec<RRMTFrameType>
}

impl WriteCommandResponder {
    pub fn new(oneshot: Responder<RRMTFrame, ReadError>, expected_types: Vec<RRMTFrameType>) -> WriteCommandResponder {
        WriteCommandResponder {
            oneshot,
            expected_types
        }
    }
}

pub struct WriteCommand {
    frame: RRMTFrame,
    response: Option<WriteCommandResponder>
}

impl WriteCommand {
    pub fn new(frame: RRMTFrame, response: Option<WriteCommandResponder>) -> WriteCommand {
        WriteCommand {
            frame,
            response
        }
    }
}

pub struct ReadCommand {
    expected_types: Vec<RRMTFrameType>,
    response: Responder<RRMTFrame, ReadError>
}

impl ReadCommand {
    pub fn new(expected_types: Vec<RRMTFrameType>, response: Responder<RRMTFrame, ReadError>) -> ReadCommand {
        ReadCommand {
            expected_types,
            response
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionRole {
    Server,
    Client
}

#[derive(Debug, Clone)]
pub struct Connection {
    machine_id: Uuid,
    write_tx: Sender<WriteCommand>,
    read_tx: Sender<ReadCommand>,
    authorized: bool,
    role: ConnectionRole
}

impl Connection {
    pub fn new(tcp_stream: TcpStream, machine_id: Uuid, role: ConnectionRole) -> Connection {
        let (write_tx, write_rx) = mpsc::channel::<WriteCommand>(10);
        let (read_tx, read_rx) = mpsc::channel::<ReadCommand>(10);


        let connection = Connection {
            machine_id,
            write_tx,
            read_tx,
            authorized: false,
            role
        };
        
        Self::spawn_managers(&connection, tcp_stream, read_rx, write_rx);
        
        connection
    }
    
    pub async fn authorize(&self) -> Result<(), AuthorizeError> {
        if self.authorized {return Err(AuthorizeError::AlreadyAuthorized)}
        
        let frame = match RRMTFrame::new_authorize(&self.role, &self.machine_id) {
            Ok(frame) => frame,
            Err(_) => return Err(AuthorizeError::SendError)
        };
        let expected_types = vec![RRMTFrameType::Denied, RRMTFrameType::Accepted, RRMTFrameType::Error];
        
        let resp = self.connection_write_reply(frame, expected_types).await;
        let x = resp.await;
        println!("{:?}", x);
        
        Ok(())
    }
    
    pub async fn connection_write(&self, frame: Result<RRMTFrame, RRMTInvalidRoleError>) {
        if let Ok(frame) = frame {
            let _ = self.write_tx.send(WriteCommand::new(frame, None)).await;
        }
    }
    
    async fn connection_read(&self, expected_types: Vec<RRMTFrameType>, response: Responder<RRMTFrame, ReadError>) -> Result<(), SendError<ReadCommand>> {
        self.read_tx.send(ReadCommand::new(expected_types, response)).await
    }
    
    fn spawn_managers(&self, socket: TcpStream, mut read_rx: Receiver<ReadCommand>, mut write_rx: Receiver<WriteCommand>) {
    }
}