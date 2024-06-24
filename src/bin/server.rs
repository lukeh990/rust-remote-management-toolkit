/*
server.rs
The server manages remote devices and exposes an API to the CLI.

To-Do:
 */

use std::error;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

use rrmt_lib::flow_handler::{ConnectionType, FlowHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    println!("RRMT Server is now listening at: {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;

        // For every new connection run the handle_new_socket function
        tokio::spawn(async move {
            handle_new_socket(socket).await;
        });
    }
}

async fn handle_new_socket(socket: TcpStream) {
    // Create flow handler
    let _flow_handler = FlowHandler::new(socket, ConnectionType::Server).await;

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
