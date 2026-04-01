use std::{
    env,
    fs::{self, File},
    path::Path,
    process::{Command, exit},
};

use anyhow::anyhow;
use noport_lib::{communication::find_socket, store::Store};
use paris::{error, info, success, warn};
use tokio::{
    signal::{self},
    sync::mpsc::channel,
};

use crate::status::get_status;

/// Start the daemon in the foreground
pub async fn start_foreground(
    store: Store,
    mut port: u16,
    https: bool,
) -> Result<(), anyhow::Error> {
    if https && port != 443 {
        warn!("You can only use port 443 with HTTPS");
        port = 443;
    }

    let tld = store.get_tld();
    let (shutdown_tx, mut shutdown_rx) = channel(1);
    info!(
        "Starting the daemon proxy server (port={}, tld={})",
        port, tld
    );

    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                shutdown_tx_clone.send(()).await.unwrap();
            }
            Err(e) => {
                error!("error in the ctrl_c signal {}", e);
            }
        }
    });

    tokio::spawn(async move {
        match shutdown_rx.recv().await {
            Some(_) => {
                if let Ok(path) = find_socket() {
                    info!("stopping the socket {}", path);
                    if let Err(e) = fs::remove_file(path) {
                        error!("error while deleting the socket {}", e);
                    }
                } else {
                    error!("Could not find a socket to delete ??");
                }

                exit(0);
            }
            None => {
                error!("got None in the shutdown_rx?");
            }
        }
    });

    let addr = format!("127.0.0.1:{}", port);
    let result = daemon::daemon::start_deamon(store, addr, https, shutdown_tx).await;

    if let Err(e) = result {
        return Err(anyhow!("error starting the daemon {}", e));
    }

    Ok(())
}

/// Start the daemon in the background
/// Will not launch another daemon if one is already running
/// Stores the process id in the store (in ~/.noport/daemon.pid)
pub async fn start_background() -> Result<(), anyhow::Error> {
    if let Ok(()) = get_status().await {
        warn!("Daemon already running");
        exit(1);
    }

    let exe_path = env::current_exe()?;

    // print the stdout and stderr to a file
    let home_dir = env::home_dir().unwrap();
    let root_folder = home_dir.join(".noport");
    let log_path = Path::new(&root_folder).join("daemon.log");
    let error_path = Path::new(&root_folder).join("daemon.error");

    let root_folder_exists = Path::new(&root_folder).exists();
    if !root_folder_exists {
        fs::create_dir(root_folder)?;
    }

    let log_file = File::create(log_path)?;
    let error_file = File::create(error_path)?;

    let args = vec!["start", "--foreground"];

    let child = Command::new(exe_path)
        .args(args)
        .stdout(log_file)
        .stderr(error_file)
        .spawn()?;

    let pid = child.id();

    success!("Daemon running on {} (PID: {})", ":2828", pid.to_string());

    Ok(())
}
