use std::process;

use noport_lib::store::Store;
use paris::{error, success, warn};

/// Stop the daemon
/// Will crash if the daemon is not running
pub fn stop_daemon(store: Store) -> Result<(), anyhow::Error> {
    let daemon_id = store.get_daemon_process_id();

    if let Ok(id) = daemon_id {
        let result = process::Command::new("kill").arg(id.to_string()).output();

        if let Err(e) = result {
            error!("Error while killing the daemon {}", e);
            return Err(anyhow::anyhow!(e));
        }

        success!("NoPort daemon stopped with success");

        return store.remove_daemon_process_id();
    } else {
        warn!("NoPort daemon is not running (cannot stop it then)");
    }

    Ok(())
}
