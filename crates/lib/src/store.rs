use std::sync::Arc;

use crate::machines::Machine;
use anyhow::{Ok, Result};
use nix::unistd::Uid;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEntry {
    pub port: i32,
    pub domain: String,
    pub path: String,
    pub subdomain: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Store<M: Machine + 'static> {
    inner: Arc<Mutex<Vec<StoreEntry>>>,
    tld: String,
    machine: &'static M,
}

pub trait NoPortStore<M: Machine + 'static> {
    fn add_host(&self, entry: StoreEntry) -> impl Future<Output = Result<()>>;

    /// Get the host for the given server (complete host without port)
    fn get_host(&self, server: String) -> impl Future<Output = Option<StoreEntry>>;

    /// Get all the hosts for the given subdomain
    fn get_hosts_for_subdomain(
        &self,
        subdomain: String,
    ) -> impl Future<Output = Result<Vec<StoreEntry>>>;

    fn get_machine(&self) -> &'static M;
}

impl<M> NoPortStore<M> for Store<M>
where
    M: Machine + 'static,
{
    async fn add_host(&self, entry: StoreEntry) -> Result<()> {
        let mut inner = self.inner.lock().await;

        inner.push(entry.clone());

        drop(inner);

        if Uid::current().is_root() {
            self.machine.add_host(entry).await?;
        }

        Ok(())
    }

    async fn get_host(&self, server: String) -> Option<StoreEntry> {
        let store = self.inner.lock().await;

        let tld = format!(".{}", self.tld);
        let sub_domain = server.replace(tld.as_str(), "");

        if let Some(entry) = store.iter().find(|e| e.domain == sub_domain) {
            return Some(entry.clone());
        }

        None
    }

    async fn get_hosts_for_subdomain(&self, subdomain: String) -> Result<Vec<StoreEntry>> {
        Ok(vec![])
    }

    fn get_machine(&self) -> &'static M {
        self.machine
    }
}
