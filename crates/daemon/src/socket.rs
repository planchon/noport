use std::io::{BufRead, Read};

use noport_lib::{
    communication::{NoPortCommunication, get_socket},
    store::{Store, StoreEntry},
};
use paris::{error, info, success};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::mpsc::Sender,
};

/// Create the socket for the client <--> daemon communication
pub async fn create_socket(store: &Store, shutdown_tx: Sender<()>) -> Result<(), anyhow::Error> {
    let socket_path = get_socket();
    let listener = UnixListener::bind(socket_path)?;

    success!("socket started (path={})", socket_path);

    while let Ok((mut stream, _)) = listener.accept().await {
        let store_clone = store.clone();
        let shutdown_clone = shutdown_tx.clone();

        tokio::spawn(async move {
            handle_connection(stream, &store_clone, shutdown_clone).await;
        });
    }

    Ok(())
}

async fn send_ok(mut stream: UnixStream) {
    let ok = serde_json::to_string(&NoPortCommunication::Ok).unwrap();
    if let Err(e) = stream.write(ok.as_bytes()).await {
        error!("error while sending OK {}", e);
    } else {
        info!("OK sent");
    }
}

async fn handle_connection(
    mut stream: UnixStream,
    store: &Store,
    shutdown_tx: Sender<()>,
) -> Result<(), anyhow::Error> {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(0) => {}
        Ok(n) => {
            let communication: NoPortCommunication = serde_json::from_slice(&buffer[..n])?;

            match communication {
                NoPortCommunication::CreateHost { domain, port, path } => {
                    info!("[comms] adding a host ({}, {}, {})", domain, port, path);
                    let mut inner = store.inner.lock().await;
                    inner.push(StoreEntry {
                        domain: domain.clone(),
                        port,
                        path,
                    });
                    send_ok(stream).await;
                    info!("[comms] entry add! ({})", domain);
                }
                NoPortCommunication::Stop => {
                    info!("[comms] stopping the daemon");
                    send_ok(stream).await;
                    shutdown_tx.send(()).await.unwrap();
                }
                NoPortCommunication::Status => {
                    info!("[comms] getting status");
                    send_ok(stream).await;
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
