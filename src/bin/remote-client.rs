/*
remote-client.rs
This file houses the remote client. It authenticates with the server and runs remote commands.

To-Do:
- [X] Connect to server
- [X] Send frames between server and remote
- [ ] Store persistent config
- [ ] Authenticate with remote server
- [ ] If already provisioned reauthenticate
- [ ] Ping/Pong Cycle
- [ ] Execute commands
 */

use std::net::SocketAddr;
use std::str::FromStr;

use uuid::Uuid;

use rrmt_lib::connection::Connection;
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
    let machine_id = Uuid::from_str("c10afcef-0d32-4b6a-a870-54318fdcef18")?;

    let mut connection = Connection::new(addr, machine_id).await?;

    match connection.authorize().await {
        Ok(_) => println!("Authorized!"),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
