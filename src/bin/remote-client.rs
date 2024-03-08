use std::net::SocketAddr;
use std::ops::Mul;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;
use rrmt_lib::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup debug variable
    let mut debug = false;
    #[cfg(debug_assertions)]
    {
        debug = true;
    }

    let addr = SocketAddr::from_str("127.0.0.1:6000")?;
    let mut stream = establish_stream_backoff(addr).await;

    stream.write_all(&[0x01]).await?;

    Ok(())
}

async fn establish_stream_backoff(addr: SocketAddr) -> TcpStream {
    let mut backoff = Duration::from_millis(1000);
    loop {
        sleep(backoff).await;
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                return stream;
            },
            Err(_) => {
                backoff.mul(2);
                println!("Failed to connect retrying after {} seconds", backoff.as_secs());
                continue;
            },
        };
    }
}