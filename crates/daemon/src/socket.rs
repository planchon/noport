use std::io::{BufRead, Read};

use noport_lib::{communication::NoPortCommunication, store::Store};
use paris::{error, info};
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::{UnixListener, UnixStream},
};

const SOCKET_PATH: &str = "/var/run/noport.socket";

/// Create the socket for the client <--> daemon communication
pub async fn create_socket(store: &Store) -> Result<(), anyhow::Error> {
    let listener = UnixListener::bind(SOCKET_PATH)?;

    info!("socket started {}", SOCKET_PATH);

    for (stream, _) in listener.accept().await {
        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }

    Ok(())
}

async fn handle_connection(mut stream: UnixStream) -> Result<(), anyhow::Error> {
    info!("new socket connection");

    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let communication: NoPortCommunication = serde_json::from_slice(&buffer[..n])?;

                match communication {
                    NoPortCommunication::CreateHost { domain, port, path } => {
                        info!("adding a host ({}, {}, {})", domain, port, path);
                    }
                    NoPortCommunication::Stop => {
                        info!("stopping the daemon");
                    }
                    NoPortCommunication::Status => {
                        info!("getting status");
                    }
                    NoPortCommunication::RemoveHost { domain } => {
                        info!("removing a host ({})", domain);
                    }
                    NoPortCommunication::Ok => {}
                }
            }
            Err(e) => {
                error!("error while reading the socket data {:?}", e);
            }
        }
    }

    Ok(())
}
