use std::fmt::format;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use paris::error;
use paris::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::hosts::write_host;
use crate::store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEntry {
    pub port: i32,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Store {
    inner: Arc<Mutex<Vec<StoreEntry>>>,
    tld: String,

    root_folder: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let home_folder = Path::new("/tmp/.noport").to_path_buf();

        if !fs::exists(&home_folder).unwrap() {
            info!(
                "Creating the .noport folder ({})",
                home_folder.to_string_lossy()
            );
            fs::create_dir(&home_folder).unwrap();
        }

        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            root_folder: home_folder,
            tld: "localhost".to_string(),
        }
    }

    pub fn root_folder(&self) -> PathBuf {
        self.root_folder.clone()
    }

    /// Set the global TLD
    pub fn set_tld(&mut self, tld: String) -> Result<(), anyhow::Error> {
        self.tld = tld;

        Ok(())
    }

    /// Get the global TLD
    pub fn get_tld(&self) -> String {
        self.tld.clone()
    }

    pub async fn add_entry(&self, entry: StoreEntry) {
        let mut inner = self.inner.lock().await;

        inner.push(entry.clone());

        drop(inner);

        self.update_hosts().await;
    }

    async fn update_hosts(&self) {
        let store_inner = self.inner.lock().await;
        let hosts = store_inner
            .iter()
            .map(|f| format!("127.0.0.1 {}.{}", f.domain.clone(), self.get_tld()))
            .collect();

        if let Err(e) = write_host(hosts) {
            error!("error while adding host {}", e);
        }
    }

    /// Daemon land
    /// Resolve the reverse proxy call
    /// Example: api.localhost -> StoreEntry { port: , domain: "api.localhost", path: "" }
    pub async fn reverse_proxy(&self, host: String) -> Option<StoreEntry> {
        let store = self.inner.lock().await;

        let tld = format!(".{}", self.get_tld());
        let sub_domain = host.replace(tld.as_str(), "");

        if let Some(entry) = store.iter().find(|e| e.domain == sub_domain) {
            return Some(entry.clone());
        }

        None
    }
}
