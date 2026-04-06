use std::path::PathBuf;

use anyhow::Result;

use crate::store::Host;

/// Interface between the OS for file, hosts and socket management
pub trait Machine {
    fn get_home() -> Result<PathBuf>;
    fn get_ca_folder() -> Result<PathBuf>;

    fn add_host(&self, host: Host) -> impl Future<Output = Result<()>> + Send;
    fn reset_hosts(&self) -> impl Future<Output = Result<()>> + Send;
}
