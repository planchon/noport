use noport_lib::client::send_command;

use noport_lib::communication::NoPortCommunication;
use tracing::{info, warn};

pub async fn get_status() -> Result<(), anyhow::Error> {
    send_command(NoPortCommunication::Status).await
}

pub async fn status() -> Result<(), anyhow::Error> {
    if let Err(e) = get_status().await {
        warn!("Daemon not running ({})", e);
    } else {
        info!("Daemon running !");
    }

    Ok(())
}
