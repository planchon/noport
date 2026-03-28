use paris::error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixSocket, UnixStream},
};

use crate::communication::{NoPortCommunication, find_socket};

pub async fn send_ok(mut stream: UnixStream) {
    let ok = serde_json::to_string(&NoPortCommunication::Ok).unwrap();
    if let Err(e) = stream.write(ok.as_bytes()).await {
        error!("error while sending OK {}", e);
    }
}

pub async fn send_command(command: NoPortCommunication) -> Result<(), anyhow::Error> {
    let socket_path_res = find_socket();
    if socket_path_res.is_err() {
        error!("Could not find the running socket");
        return Err(socket_path_res.err().unwrap());
    }
    let socket_path = socket_path_res.unwrap();

    let socket = UnixSocket::new_stream()?;

    let mut stream = socket.connect(socket_path).await?;

    let command_clone = command.clone();

    let bytes = serde_json::to_string(&command_clone)?;
    // send the command
    stream.write(bytes.as_bytes()).await?;

    // wait for the response
    let mut buffer = [0; 1024];
    let size = stream.read(&mut buffer).await?;

    let communication: NoPortCommunication = serde_json::from_slice(&buffer[..size])?;

    Ok(())
}
