/*
connection.rs
This module provides a high level interface for interacting with the RRMT protocol.

To-Do:
- [ ] 
 */

use tokio::net::TcpStream;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        Connection {
            stream
        }
    }
}