use std::process::{Command, ExitStatus, Stdio};

/// Start a subprocess and return the command and the stdin/stdout/stderr pipes
pub fn start(args: Vec<String>) -> Option<ExitStatus> {
    if args.is_empty() {
        return None;
    }

    // start the subprocess
    let main_command = args[0].clone();
    let main_args = args[1..].to_vec();

    let status = Command::new(main_command)
        .args(main_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to start subprocess");

    Some(status)
}
