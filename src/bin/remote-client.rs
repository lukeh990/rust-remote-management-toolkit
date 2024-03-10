use std::net::SocketAddr;
use std::ops::MulAssign;
use std::str::FromStr;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;
use uuid::Uuid;

use rrmt_lib::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup debug variable
/*    let mut debug = false;
    #[cfg(debug_assertions)]
    {
        debug = true;
    }
*/

    let addr = SocketAddr::from_str("127.0.0.1:3000")?;
    let mut stream = establish_stream_backoff(addr).await;

    let token = Uuid::from_str("c10afcef-0d32-4b6a-a870-54318fdcef18")?;
    let mut to_send = vec![0x02, 0x00, 0x10];
    to_send.extend_from_slice(token.as_bytes());
    
    stream.write_all(&to_send).await?;

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
                backoff.mul_assign(2);
                println!("Failed to connect retrying after {} seconds", backoff.as_secs());
                continue;
            },
        };
    }
}