use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_request(
    stream: TcpStream,
    store: Arc<Mutex<HashMap<String, u16>>>,
) -> io::Result<()> {
    println!("Handling request");

    Ok(())
}
