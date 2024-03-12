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

use std::collections::HashSet;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;
use rrmt_lib::frame::{read_frame, RRMTFrame, write_frame};

use rrmt_lib::Result;

type SharedTokenList = Arc<Mutex<HashSet<Uuid>>>;

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

    loop {
        let (socket, _) = listener.accept().await?;
        let shared_token_list = shared_token_list.clone();

        tokio::spawn(async move {
            if let Err(e) = process(socket, shared_token_list).await {
                println!("Failure: {}", e);
            }
        });
    }
}

async fn process(mut socket: TcpStream, shared_token_list: SharedTokenList) -> Result<()> {
    let frame = match read_frame(&mut socket).await {
        Ok(frame) => frame,
        Err(e) => return write_frame(&mut socket, RRMTFrame::Error(e.to_string())).await,
    };

    Ok(())
}