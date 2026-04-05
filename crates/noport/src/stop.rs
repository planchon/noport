use noport_lib::client::send_command;
use noport_lib::communication::NoPortCommunication;
use tracing::{error, info};

/// Stop the daemon
/// Will crash if the daemon is not running
pub async fn stop_daemon() -> Result<(), anyhow::Error> {
    if let Err(e) = send_command(NoPortCommunication::Stop).await {
        error!("could not send the stop command {}", e);
    } else {
        info!("Daemon stopped");
    }

    Ok(())
}
