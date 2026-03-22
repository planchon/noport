use noport_lib::store::Store;
use paris::{success, warn};

pub fn status(store: Store) -> Result<(), anyhow::Error> {
    if let Ok(pid) = store.get_daemon_process_id() {
        success!("NoPort daemon is running");
        success!("Running on PID: {}", pid);
    } else {
        warn!("NoPort daemon is not running");
    }

    Ok(())
}
