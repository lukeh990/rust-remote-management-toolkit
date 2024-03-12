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

use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use rrmt_lib::frame::{read_frame, ReadErrorType, RRMTFrame, write_frame};
use rrmt_lib::Result;

type SharedTokenList = Arc<Mutex<HashSet<Uuid>>>;
type SharedRemoteList = Arc<Mutex<HashMap<Uuid, bool>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    println!("RRMT Server is now listening at: {}", addr);

    let token = Uuid::from_str("c10afcef-0d32-4b6a-a870-54318fdcef18")?;
    println!("UUID is {}", token);

    let mut token_list = HashSet::new();
    token_list.insert(token);

    let shared_token_list: SharedTokenList = Arc::new(Mutex::new(token_list));
    let shared_remote_list: SharedRemoteList = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await?;
        let shared_token_list = shared_token_list.clone();
        let shared_remote_list = shared_remote_list.clone();

        tokio::spawn(async move {
            if let Err(e) = process(socket, shared_token_list, shared_remote_list).await {
                println!("Failure: {}", e);
            }
        });
    }
}

async fn process(mut socket: TcpStream, shared_token_list: SharedTokenList, shared_remote_list: SharedRemoteList) -> Result<()> {
    loop {
        let frame = match read_frame(&mut socket).await {
            Ok(frame) => frame,
            Err(e) => {
                match e.error_type {
                    ReadErrorType::ClientError => {
                        write_frame(&mut socket, RRMTFrame::Error(e.to_string())).await?;
                        continue;
                    }
                    ReadErrorType::ConnectionError => break
                };
            }
        };

        match frame {
            RRMTFrame::Authorize(uuid) => {
                let mut token_list: HashSet<Uuid> = HashSet::new();
                {
                    match shared_token_list.lock() {
                        Ok(shared_token_list) => shared_token_list.clone_into(&mut token_list),
                        Err(_) => return Err("Token Mutex Poison".into()),
                    };
                }

                if token_list.contains(&uuid) {
                    let uuid = Uuid::new_v4();
                    let mut remote_list: HashMap<Uuid, bool> = HashMap::new();
                    {
                        match shared_remote_list.lock() {
                            Ok(shared_remote_list) => shared_remote_list.clone_into(&mut remote_list),
                            Err(_) => return Err("Remote Mutex Poison".into())
                        };
                    }    
                    
                    write_frame(&mut socket, RRMTFrame::Provision(uuid)).await?;
                } else {
                    write_frame(&mut socket, RRMTFrame::Denied("Invalid Token".to_string()))
                        .await?;
                }
            },

            _ => println!("Unhandled frame: {:?}", frame),
        };
    }

    Err("Socket Broken".into())
}
