/*
remote-client.rs
This file houses the remote client. It authenticates with the server and runs remote commands.

To-Do:
 */

use std::net::SocketAddr;
use std::str::FromStr;
use std::error;

use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error>> {
    let addr = SocketAddr::from_str("127.0.0.1:3000")?;



    Ok(())
}
