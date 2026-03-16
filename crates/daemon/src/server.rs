use std::io;

use tokio::net::TcpStream;

pub async fn handle_request(stream: TcpStream) -> io::Result<()> {
    println!("Handling request");

    Ok(())
}
