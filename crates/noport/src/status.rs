use noport_lib::{client::send_command, store::Store};
use paris::{error, success, warn};

use noport_lib::communication::NoPortCommunication;

pub async fn status() -> Result<(), anyhow::Error> {
    if let Err(e) = send_command(NoPortCommunication::Status).await {
        warn!("Daemon not running ({})", e);
        return Ok(());
    }

    success!("Daemon running !");
    Ok(())
}
