use noport_lib::store::Store;
use paris::info;

pub fn status(store: Store) -> Result<(), anyhow::Error> {
    if let Ok(pid) = store.get_daemon_process_id() {
        info!("NoPort daemon is running");
        info!("Running on PID: {}", pid);
    } else {
        info!("NoPort daemon is not running");
    }

    Ok(())
}
