use paris::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixSocket,
};

use crate::communication::NoPortCommunication;

const DEFAULT_SOCKET: &str = "/var/run/noport.socket";

pub async fn send_command(command: NoPortCommunication) -> Result<(), anyhow::Error> {
    let socket = UnixSocket::new_stream()?;

    let mut stream = socket.connect(DEFAULT_SOCKET).await?;

    let command_clone = command.clone();

    let bytes = serde_json::to_string(&command_clone)?;
    // send the command
    stream.write(bytes.as_bytes()).await?;

    // wait for the response
    let mut buffer = [0; 1024];
    let size = stream.read(&mut buffer).await?;

    let communication: NoPortCommunication = serde_json::from_slice(&buffer[..size])?;

    info!("received {:?}", communication);

    Ok(())
}
