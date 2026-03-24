use paris::{error, info};
use std::{
    env,
    process::{Command, ExitStatus, Stdio},
};

use crate::{
    client::send_command, communication::NoPortCommunication, domain::generate_domain,
    port::find_free_port,
};

/// Start a subprocess and return the command and the stdin/stdout/stderr pipes
pub async fn start(args: Vec<String>) -> Option<ExitStatus> {
    if args.is_empty() {
        return None;
    }

    let port = find_free_port().await.unwrap();
    let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    let domain = generate_domain(&current_dir).unwrap();

    let command = NoPortCommunication::CreateHost {
        domain: domain.clone(),
        port,
        path: current_dir,
    };

    if let Err(e) = send_command(command).await {
        error!("could not register the host {}", e);
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
