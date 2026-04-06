use std::{
    env,
    ffi::CString,
    os::unix::fs::chown,
    process::{Command, Stdio},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use nix::{libc::chmod, unistd::Gid};
use noport_lib::{
    client::send_ok,
    communication::{NoPortCommunication, get_socket},
    linux::{add_user_to_group, get_user, upsert_group},
    machines::Machine,
    store::{Store, StoreEntry},
};
use tokio::{
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
    sync::mpsc::Sender,
};
use tracing::{debug, error, info};

const GROUP_NAME: &str = "noport";

fn macos_rights() -> Result<Gid> {
    info!("Settings all the MacOS rights");

    let group = format!("/Groups/{}", GROUP_NAME);
    let user = get_user();
    Command::new("dscl")
        .args(vec![".", "create", &group, "PrimaryGroupID", "121212"])
        .stderr(Stdio::inherit())
        .status()?;

    let user_name = user.name.as_str();
    Command::new("dseditgroup")
        .args(vec![
            "-o", "edit", "-a", user_name, "-t", "user", GROUP_NAME,
        ])
        .stderr(Stdio::inherit())
        .status()?;

    Ok(Gid::from(121212))
}

fn linux_rights() -> Result<Gid> {
    info!("Setting all the linux rights");

    // !!should be done at install
    let group = upsert_group(GROUP_NAME)?;
    let user = get_user();
    add_user_to_group(user, &group)?;

    return Ok(group.gid);
}

fn ensure_socket_right(socket_path: &str) -> Result<()> {
    let gid = match env::consts::OS {
        "macos" => macos_rights(),
        "linux" => linux_rights(),
        _ => {
            return Err(anyhow::Error::msg("OS not supported"));
        }
    }?;

    // set noport as the socket groups
    if let Err(e) = chown(socket_path, None, Some(gid.as_raw())) {
        error!("error while chown-ing the socket ({}): {}", socket_path, e);
        return Err(anyhow::Error::from(e));
    }

    // change the group right
    let c_path = CString::new(socket_path).unwrap();
    unsafe {
        chmod(c_path.as_ptr(), 0o775);
    }

    Ok(())
}

/// Create the socket for the client <-> daemon communication
pub async fn create_socket<M: Machine + 'static>(
    store: &Store<M>,
    shutdown_tx: Sender<()>,
) -> Result<()> {
    let socket_path = get_socket();
    let listener = UnixListener::bind(socket_path)?;

    let current_user = nix::unistd::Uid::current();
    if current_user.is_root() {
        if let Err(e) = ensure_socket_right(socket_path) {
            error!("error while setting socket perms {}", e);
            return Err(anyhow!(e));
        }
    }

    info!("socket started (path={})", socket_path);

    while let Ok((stream, _)) = listener.accept().await {
        let store_clone = store.clone();
        let shutdown_clone = shutdown_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &store_clone, shutdown_clone).await {
                error!("Error while handling the connection {}", e);
            };
        });
    }

    Ok(())
}

async fn handle_connection<M: Machine>(
    mut stream: UnixStream,
    store: &Store<M>,
    shutdown_tx: Sender<()>,
) -> Result<()> {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(0) => {}
        Ok(n) => {
            let communication: NoPortCommunication = serde_json::from_slice(&buffer[..n])?;

            match communication {
                NoPortCommunication::CreateHost { domain, port, path } => {
                    store
                        .add_entry(StoreEntry {
                            port,
                            domain: domain.clone(),
                            path: path.clone(),
                        })
                        .await;
                    send_ok(stream).await;
                    debug!("[comms] host add ({}, {}, {})", domain, port, path);
                }
                NoPortCommunication::Stop => {
                    send_ok(stream).await;
                    debug!("[comms] stopping the daemon");
                    shutdown_tx.send(()).await.unwrap();
                }
                NoPortCommunication::Status => {
                    debug!("[comms] getting tatus");
                    send_ok(stream).await;
                }
                NoPortCommunication::RemoveHost { domain } => {
                    debug!("[comms] removing a host ({})", domain);
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
