use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Result, bail};
use rand::{RngExt, rng};
use tokio::{
    fs::{create_dir, try_exists},
    net::TcpStream,
};
use tracing::{info, warn};

use crate::{
    machines::{
        BasicMachine, HOST_FILE_BEGIN_MARKER, HOST_FILE_END_MARKER, Machine, MachineErrors,
    },
    store::StoreEntry,
};

pub struct LinuxMachine {
    basic_machine: BasicMachine,
}

impl LinuxMachine {
    pub async fn new() -> Result<Self, MachineErrors> {
        let bm = LinuxMachine::basic_machine().await?;

        Ok(Self { basic_machine: bm })
    }
}

impl Machine for LinuxMachine {
    async fn basic_machine() -> Result<BasicMachine, MachineErrors> {
        let home = env::home_dir();

        if home.is_none() {
            return Err(MachineErrors::NoHome);
        }

        let home_dir = home.unwrap();
        let ca_dir = home_dir.join(".noport/ca");
        let certs_dir = home_dir.join(".noport/certs");

        if !try_exists(&home_dir).await? {
            info!(
                "Creating the .noport folder ({})",
                home_dir.to_string_lossy()
            );

            create_dir(home_dir.join(".noport")).await?;
        }

        if !try_exists(&ca_dir).await? {
            info!("Creating the CA folder ({})", ca_dir.to_string_lossy());
            create_dir(&ca_dir).await?;
        }

        if !try_exists(&certs_dir).await? {
            info!(
                "Creating the certs folder ({})",
                certs_dir.to_string_lossy()
            );
            create_dir(&certs_dir).await?;
        }

        Ok(BasicMachine {
            home_dir,
            ca_dir,
            certs_dir,
        })
    }

    fn get_home(&self) -> PathBuf {
        self.basic_machine.home_dir.clone()
    }

    fn get_ca_folder(&self) -> PathBuf {
        self.basic_machine.ca_dir.clone()
    }

    fn get_certs_folder(&self) -> PathBuf {
        self.basic_machine.certs_dir.clone()
    }

    fn user_is_privileged() -> Result<(), MachineErrors> {
        let uid = nix::unistd::Uid::current();

        if uid.is_root() {
            Ok(())
        } else {
            Err(MachineErrors::NotRoot)
        }
    }

    async fn find_port(&self) -> Result<u16> {
        let mut attempts = 0;
        // cannot be Send because of rng
        let mut rng = rng();

        loop {
            if attempts > 10 {
                bail!(MachineErrors::NoPort)
            }

            let port = rng.random_range(10000..65535);

            let socket = format!("127.0.0.1:{}", port);
            let stream = TcpStream::connect(socket);

            if let Err(_) = tokio::time::timeout(Duration::from_secs(5), stream).await {
                attempts += 1;
                continue;
            } else {
                return Ok(port);
            }
        }
    }

    async fn add_host(&self, entry: StoreEntry) -> Result<(), MachineErrors> {
        Self::user_is_privileged()?;

        let linux_host_file = Path::new("/etc/hosts");

        let file = tokio::fs::read_to_string(linux_host_file).await?;

        let mut content = file
            .split("\n")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let begin_index = content.iter().position(|s| s == HOST_FILE_BEGIN_MARKER);
        let end_index = content.iter().position(|s| s == HOST_FILE_END_MARKER);

        if begin_index.is_some() && end_index.is_some() {
            let host = if entry.subdomain.is_some() {
                format!("{}.{}", entry.subdomain.unwrap(), entry.domain)
            } else {
                entry.domain
            };

            let new_host = format!("{}.{} 127.0.0.1", host, "localhost");
            content.insert(end_index.unwrap(), new_host);
        }

        let new_file = content.join("\n");

        tokio::fs::write(linux_host_file, new_file).await?;

        Ok(())
    }

    async fn reset_hosts(&self) -> Result<(), MachineErrors> {
        Self::user_is_privileged()?;

        let linux_host_file = Path::new("/etc/hosts");

        let file = tokio::fs::read_to_string(linux_host_file).await?;

        let mut content = file
            .split("\n")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let begin_index = content.iter().position(|s| s == HOST_FILE_BEGIN_MARKER);
        let end_index = content.iter().position(|s| s == HOST_FILE_END_MARKER);

        if begin_index.is_none() || end_index.is_none() {
            return Ok(());
        }

        if begin_index.is_none() || end_index.is_none() {
            warn!("the host file is not well formatted. skipping");
            return Ok(());
        }

        content.drain(begin_index.unwrap()..end_index.unwrap());

        let new_file = content.join("\n");

        tokio::fs::write(linux_host_file, new_file).await?;

        Ok(())
    }
}
