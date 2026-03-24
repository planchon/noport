use std::{
    env,
    fs::{self, File},
    path::Path,
    process::Command,
};

use noport_lib::store::Store;
use paris::{error, info, success, warn};
use tokio::{
    runtime::Runtime,
    signal::{self, unix::signal},
    sync::mpsc::channel,
};

/// Start the daemon in the foreground
pub async fn start_foreground(store: Store) -> Result<(), anyhow::Error> {
    let tld = store.get_tld();
    let (shutdown_tx, shutdown_rx) = channel(1);
    info!(
        "Starting the daemon proxy server (port={}, tld={})",
        "2828", tld
    );

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => shutdown_tx.send(()).await.unwrap(),
            Err(e) => {
                error!("error in the ctrl_c signal {}", e);
            }
        }
    });

    let result = daemon::daemon::start_deamon(store, None, shutdown_rx).await;

    if let Err(e) = result {
        error!("Error starting the daemon: {}", e);
        return Ok(());
    }

    Ok(())
}

/// Start the daemon in the background
/// Will not launch another daemon if one is already running
/// Stores the process id in the store (in ~/.noport/daemon.pid)
pub fn start_background(store: Store) -> Result<(), anyhow::Error> {
    if let Ok(process_id) = store.get_daemon_process_id() {
        warn!("Daemon is already running");
        warn!("Running on PID: <i>{}</i>", process_id.clone().to_string());
        return Ok(());
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
    store.set_daemon_process_id(pid)?;

    success!("Daemon running on {} (PID: {})", ":2828", pid.to_string());

    Ok(())
}
