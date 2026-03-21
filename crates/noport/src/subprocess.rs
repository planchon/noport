use std::{
    env,
    process::{Command, ExitStatus, Stdio},
};

use crate::{domain::generate_domain, port::find_free_port, store::Store};

/// Start a subprocess and return the command and the stdin/stdout/stderr pipes
pub async fn start(args: Vec<String>, store: Store) -> Option<ExitStatus> {
    if args.is_empty() {
        return None;
    }

    let port = find_free_port().await.unwrap();
    let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    let domain = generate_domain(&current_dir).unwrap();

    // register the new element to the store
    if let Err(e) = store
        .add_proxy_entry(current_dir.clone(), domain, port)
        .await
    {
        println!("Error while registering the process {:?}", e);
        return None;
    }

    // start the subprocess
    let main_command = args[0].clone();
    let main_args = args[1..].to_vec();

    let status = Command::new(main_command.clone())
        .args(main_args)
        .env("PORT", port.to_string())
        .env("HOST", "127.0.0.1")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to start subprocess");

    println!("Running: {} on port={}", main_command, port);

    Some(status)
}
