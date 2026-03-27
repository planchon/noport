use noport_lib::client::send_command;
use paris::{success, warn};

use noport_lib::communication::NoPortCommunication;

pub async fn get_status() -> Result<(), anyhow::Error> {
    send_command(NoPortCommunication::Status).await
}

pub async fn status() -> Result<(), anyhow::Error> {
    if let Err(e) = get_status().await {
        warn!("Daemon not running ({})", e);
    } else {
        success!("Daemon running !");
    }

    Ok(())
}
