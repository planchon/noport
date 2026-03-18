use std::io;

use tokio::net::TcpStream;

use noport_lib::store::Store;

pub async fn handle_request(stream: TcpStream, store: Store) -> io::Result<()> {
    println!("Handling request");

    Ok(())
}
