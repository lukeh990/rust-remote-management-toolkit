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
use crate::frame::{ReadError, RRMTFrame, RRMTFrameType, RRMTRole};

type Responder<T, E> = oneshot::Sender<Result<T, E>>;

pub struct WriteCommandResponder {
    oneshot: Responder<RRMTFrame, ReadError>,
    expected_types: Vec<RRMTFrameType>
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

async fn spawn_managers(socket: TcpStream) {
    let (mut read, mut write) = socket.into_split();
    
    let (write_tx, mut write_rx) = mpsc::channel::<WriteCommand>(10);
    let (read_tx, mut read_rx) = mpsc::channel::<ReadCommand>(10);
    
    tokio::spawn(async move {
        loop {
            match RRMTFrame::read(&mut read).await {
                Ok(frame) => {
                    let recv = read_rx.try_recv();
                    if let RRMTFrameType::Ping = frame.frame_type {
                        let pong_frame = match RRMTFrame::new_pong(RRMTRole::Client) {
                            Ok(frame) => frame,
                            Err(_) => {println!("Failed to create pong frame"); continue;}
                        };
                        let _ = write_tx.send(WriteCommand::new(pong_frame, None)).await;
                    };
                },
                Err(_) => {
                    // Fix Immediately
                    panic!("Failed to read frame.")
                }
            };
        }
    });
    
    tokio::spawn(async move {
        while let Some(recv) = write_rx.recv().await {
            if let Err(_) = recv.frame.write(&mut write).await {
                if let Some(resp) = recv.response {
                    let _ = resp.oneshot.send(Err(ReadError::ReadFailure));
                }
            } else if let Some(resp) = recv.response {
                let (read_resp_tx, read_resp_rx) = oneshot::channel::<Result<RRMTFrame, ReadError>>();
                if read_tx.send(ReadCommand::new(resp.expected_types, read_resp_tx)).await.is_err() {
                    let _ = resp.oneshot.send(Err(ReadError::ReadFailure));
                    continue;
                }
                let read_resp = match read_resp_rx.await {
                    Ok(reply) => reply,
                    Err(_) => {let _ = resp.oneshot.send(Err(ReadError::ReadFailure)); continue;},
                };
                let _ = resp.oneshot.send(read_resp);
            }
        }
    });
} 