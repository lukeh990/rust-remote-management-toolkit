/*
server.rs
The server manages remote devices and exposes an API to the CLI.

To-Do:
 */

use std::error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn error>> {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    println!("RRMT Server is now listening at: {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_new_socket(socket).await {
                println!("Failure: {}", e);
            }
        });
    }
}

async fn handle_new_socket(socket: TcpStream) {
    let flow_handler = FlowHandler::new(socket).await;
}

struct FlowHandler {
    handler_cmd: Sender<>
}

impl FlowHandler {
    async fn new(socket: TcpStream) -> FlowHandler {
        let (tx, mut rx) = mpsc::channel(32);
        FlowHandler {
            handler_cmd: tx
        }
    }
}