use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use rrmt_lib::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:6000";
    let listener = TcpListener::bind(addr).await?;
    println!("RRMT Server is now listening at: {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = process(socket).await {
                println!("Error: {}", e.to_string());
            }
        });
    }
}

async fn process(socket: TcpStream) -> Result<()> {
    let remote_ip = socket.peer_addr()?;
    println!("connection from: {}", remote_ip.ip().to_string());

    let rrmt_type = determine_type(socket).await?;
    println!("{:X?}", rrmt_type);

    Ok(())
}

async fn determine_type(mut socket: TcpStream) -> Result<u8> {
    let mut buffer = [0; 1];

    socket.read_exact(&mut buffer[..]).await?;

    Ok(buffer[0])
}