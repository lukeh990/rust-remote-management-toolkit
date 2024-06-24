/*
remote-client.rs
This file houses the remote client. It authenticates with the server and runs remote commands.

To-Do:
 */

use std::error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::time::sleep;

use rrmt_lib::flow_handler::{ConnectionType, FlowHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let addr = SocketAddr::from_str("127.0.0.1:3000")?;

    let stream = TcpStream::connect(addr).await?;

    let _flow_handler = FlowHandler::new(stream, ConnectionType::Client).await;

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
