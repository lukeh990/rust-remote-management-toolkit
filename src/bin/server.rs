/*
server.rs
The server manages remote devices and exposes an API to the CLI.

To-Do:
- [X] Get connection from remote
- [ ] Authorization flow
- [ ] Shared state for remote list
- [ ] Find way to persist valid tokens
- [ ] Ping/Pong Cycle
- [ ] Execute commands
- [ ] HTTP API for CLI
 */

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{error, fmt};

use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use rrmt_lib::frame::{read_frame, write_frame, RRMTError, RRMTFrame, ReadError};
use rrmt_lib::Result;

type RemoteList = HashMap<Uuid, bool>;
type SharedRemoteList = Arc<Mutex<RemoteList>>;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    println!("RRMT Server is now listening at: {}", addr);

    let token = Uuid::from_str("c10afcef-0d32-4b6a-a870-54318fdcef18")?;

    let mut remote_list = HashMap::new();
    remote_list.insert(token, false);

    let shared_remote_list: SharedRemoteList = Arc::new(Mutex::new(remote_list));

    loop {
        let (socket, _) = listener.accept().await?;
        let shared_remote_list = shared_remote_list.clone();

        tokio::spawn(async move {
            if let Err(e) = process(socket, shared_remote_list).await {
                println!("Failure: {}", e);
            }
        });
    }
}

async fn process(mut socket: TcpStream, shared_remote_list: SharedRemoteList) -> Result<()> {
    let mut machine_id = Uuid::nil();

    let (mut read, mut write) = socket.into_split();
    
    loop {
        let frame = match read_frame(&mut read).await {
            Ok(frame) => frame,
            Err(e) => {
                match e {
                    ReadError::PeerError(msg) => {
                        write_frame(
                            &mut write,
                            RRMTFrame::Error(RRMTError::FormatError, msg.to_string()),
                        )
                        .await?;
                        continue;
                    }

                    ReadError::ConnectionError(msg) => {
                        write_frame(
                            &mut write,
                            RRMTFrame::Error(RRMTError::ServerError, msg.to_string()),
                        )
                        .await?;
                        continue;
                    }

                    ReadError::LengthMismatch => {
                        write_frame(
                            &mut write,
                            RRMTFrame::Error(RRMTError::LengthMismatch, "".to_string()),
                        )
                        .await?;
                        continue;
                    }

                    ReadError::SocketClosed => break,
                };
            }
        };

        match frame {
            RRMTFrame::Authorize(uuid) => {
                if let Some(value) = get_remote_list(&shared_remote_list).await?.get(&uuid) {
                    if !(*value) {
                        insert_remote_list(&shared_remote_list, uuid, true).await?;
                        machine_id = uuid;
                        write_frame(&mut write, RRMTFrame::Accepted).await?;
                        println!("Device {} has joined.", uuid);
                        continue;
                    }
                }
                write_frame(&mut write, RRMTFrame::Denied).await?;
            }

            _ => println!("Unhandled frame: {:?}", frame),
        };
    }
    

    if !machine_id.is_nil() {
        insert_remote_list(&shared_remote_list, machine_id, false).await?;
        println!("{} has left", machine_id);
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct MutexPoisonError;

impl fmt::Display for MutexPoisonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for MutexPoisonError {}

async fn get_remote_list(shared_remote_list: &SharedRemoteList) -> Result<RemoteList> {
    let mut remote_list: RemoteList = HashMap::new();

    match shared_remote_list.lock() {
        Ok(shared_token_list) => shared_token_list.clone_into(&mut remote_list),
        Err(_) => return Err(MutexPoisonError.into()),
    }

    Ok(remote_list)
}

async fn insert_remote_list(
    shared_remote_list: &SharedRemoteList,
    key: Uuid,
    value: bool,
) -> Result<()> {
    match shared_remote_list.lock() {
        Ok(mut shared_remote_list) => shared_remote_list.insert(key, value),
        Err(_) => return Err(MutexPoisonError.into()),
    };
    Ok(())
}
