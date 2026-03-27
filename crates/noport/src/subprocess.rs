use noport_lib::{
    client::send_command, communication::NoPortCommunication, domain::generate_domain,
    port::find_free_port,
};
use paris::{error, info};
use std::{
    env,
    process::{Command, ExitStatus, Stdio, exit},
    time::Duration,
};
use tokio::time::sleep;

use crate::{start::start_background, status::get_status};

/// Start a subprocess and return the command and the stdin/stdout/stderr pipes
pub async fn start_subcommand(args: Vec<String>) -> Option<ExitStatus> {
    if args.is_empty() {
        return None;
    }

    let port = find_free_port().await.unwrap();
    let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    let domain = generate_domain(&current_dir).unwrap();

    if let Err(_) = get_status().await {
        info!("The daemon is not runnning, launching it");
        start_background().await.ok()?;
        // this works but i can do better (polling?)
        // wait for the daemon to be ready
        sleep(Duration::from_millis(250)).await;
    }

    let command = NoPortCommunication::CreateHost {
        domain: domain.clone(),
        port,
        path: current_dir,
    };

    if let Err(e) = send_command(command).await {
        error!("could not register the host {}", e);
        exit(1);
    }

    // start the subprocess
    let main_command = args[0].clone();
    let mut main_args = args[1..].to_vec();

    // vite args
    let port_args = format!("--port={}", port.clone().to_string());
    let host_args = format!("--host=127.0.0.1");

    main_args.push(port_args);
    main_args.push(host_args);

    info!(
        "Running: {} on domain={} port={}",
        main_command, domain, port
    );

    let status = Command::new(main_command.clone())
        .args(main_args)
        .env("PORT", port.to_string())
        .env("HOST", "127.0.0.1")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to start subprocess");

    Some(status)
}

pub fn rerun_as_sudo() {
    let noport = env::current_exe().unwrap().to_string_lossy().to_string();
    let mut args = Vec::from_iter(env::args());
    args.splice(0..1, vec![noport]);

    info!("running {:?}", args);

    Command::new("sudo")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("could not rerun the command as sudo");
}
