use paris::{error, warn};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn add_host(host: String) -> Result<(), anyhow::Error> {
    let host_path = Path::new("/etc/hosts");
    let host_file = fs::read_to_string(host_path).unwrap();

    let exists = host_file.split('\n').any(|line| line.contains(&host));

    if exists {
        warn!("Host already registered, skipping... ({})", host);
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(host_path)
        .unwrap();

    let host_content = format!("127.0.0.1 {}\n", host);

    if let Err(e) = file.write(host_content.as_bytes()) {
        error!("Error while adding the new host {} : {}", host, e);
        return Err(anyhow::Error::new(e));
    }

    Ok(())
}
