use std::collections::HashSet;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use rrmt_lib::{frame, Result};

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
                println!("Error: {}", e);
            }
        });
    }
}

async fn process(socket: TcpStream, shared_token_list: SharedTokenList) -> Result<()> {

    let frame = frame::read_frame(socket).await?;
    println!("{:?}", frame);
    
    {
        let token_list = shared_token_list.lock().unwrap_or_else(|e| e.into_inner());

        let _ = token_list.contains(&Uuid::from_str("c10afcef-0d32-4b6a-a870-54318fdcef18")?);
    }
    
    Ok(())
}