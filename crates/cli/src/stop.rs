use std::process;

use noport_lib::store::Store;

/// Stop the daemon
/// Will crash if the daemon is not running
pub fn stop_daemon(store: Store) -> Result<(), anyhow::Error> {
    let daemon_id = store.get_daemon_process_id();

    if let Ok(id) = daemon_id {
        let result = process::Command::new("kill").arg(id.to_string()).output();

        if let Err(e) = result {
            println!("{}", e.to_string());
        }

        return store.remove_daemon_process_id();
    } else {
        Err(anyhow::anyhow!("The deamon is not running"))
    }
}
