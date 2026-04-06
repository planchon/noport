use std::path::PathBuf;

use anyhow::Result;

use crate::store::Host;

pub mod linux;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum MachineErrors {
    #[error("the command needs to be run as sudo")]
    NotRoot,
    #[error("could not find the home directory")]
    NoHome,
    #[error("could not find a free port")]
    NoPort,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

const HOST_FILE_BEGIN_MARKER: &str = "# noport begin";
const HOST_FILE_END_MARKER: &str = "# noport end";

/// Basic machine information
pub struct BasicMachine {
    pub home_dir: PathBuf,
    pub ca_dir: PathBuf,
    pub certs_dir: PathBuf,
}

/// Interface between the OS for file, hosts and socket management
pub trait Machine {
    fn setup_machine(&mut self) -> impl Future<Output = Result<(), MachineErrors>> + Send;
    fn get_home(&self) -> PathBuf;
    fn get_ca_folder(&self) -> PathBuf;
    fn get_certs_folder(&self) -> PathBuf;

    fn user_is_privileged() -> Result<(), MachineErrors>;

    fn add_host(&self, host: Host) -> impl Future<Output = Result<(), MachineErrors>> + Send;
    fn reset_hosts(&self) -> impl Future<Output = Result<(), MachineErrors>> + Send;
    fn find_port(&self) -> impl Future<Output = Result<u16>>;
}
