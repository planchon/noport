use std::io::{BufRead, Read};

use noport_lib::{
    communication::{NoPortCommunication, get_socket},
    store::Store,
};
use paris::{error, info, success};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

/// Create the socket for the client <--> daemon communication
pub async fn create_socket(store: &Store) -> Result<(), anyhow::Error> {
    let socket_path = get_socket();
    let listener = UnixListener::bind(socket_path)?;

    success!("socket started (path={})", socket_path);

    while let Ok((mut stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }

    Ok(())
}

async fn handle_connection(mut stream: UnixStream) -> Result<(), anyhow::Error> {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(0) => {}
        Ok(n) => {
            let communication: NoPortCommunication = serde_json::from_slice(&buffer[..n])?;

            match communication {
                NoPortCommunication::CreateHost { domain, port, path } => {
                    info!("[comms] adding a host ({}, {}, {})", domain, port, path);
                }
                NoPortCommunication::Stop => {
                    info!("[comms] stopping the daemon");
                }
                NoPortCommunication::Status => {
                    info!("[comms] getting status");
                    let ok = serde_json::to_string(&NoPortCommunication::Ok).unwrap();
                    if let Err(e) = stream.write(ok.as_bytes()).await {
                        error!("error while sending status {}", e);
                    } else {
                        info!("status sent");
                    }
                }
                NoPortCommunication::RemoveHost { domain } => {
                    info!("[comms] removing a host ({})", domain);
                }
                NoPortCommunication::Ok => {}
            }
        }
        Err(e) => {
            error!("error while reading the socket data {:?}", e);
        }
    }

    Ok(())
}
