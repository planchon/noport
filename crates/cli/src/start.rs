use core::error;
use std::{
    env,
    fs::{self, File},
    path::Path,
    process::Command,
};

use ansi_term::Colour;
use noport_lib::store::Store;
use tokio::runtime::Runtime;

/// Start the daemon in the foreground
pub fn start_foreground(store: Store) -> Result<(), anyhow::Error> {
    let runtime = Runtime::new().unwrap();
    println!(
        "\n{} {}\n",
        Colour::Fixed(29).paint("Starting the daemon proxy server"),
        Colour::Fixed(31).paint("(:2828)")
    );
    let result = runtime.block_on(daemon::daemon::start_deamon(store, None));

    if let Err(e) = result {
        println!("{}", Colour::Red.paint(e.to_string()));
    }

    println!("{}", Colour::Fixed(50).paint("Proxy server started"));

    Ok(())
}

/// Start the daemon in the background
/// Stores the process id in the store
pub fn start_background(store: Store) -> Result<(), anyhow::Error> {
    let exe_path = env::current_exe()?;

    println!("test");

    // print the stdout and stderr to a file
    let root_folder = store.get_root_folder();
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

    Ok(())
}
