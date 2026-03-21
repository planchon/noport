use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEntry {
    pub port: i32,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Store {
    inner: Arc<Mutex<Vec<StoreEntry>>>,

    root_folder: String,
}

impl Store {
    pub fn new() -> Self {
        let home_dir = env::home_dir().unwrap();
        let home_folder = home_dir.join(".noport").to_string_lossy().to_string();

        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            root_folder: home_folder,
        }
    }

    pub fn get_root_folder(&self) -> String {
        self.root_folder.clone()
    }

    pub async fn add_proxy_entry(
        &self,
        path: String,
        domain: String,
        port: i32,
    ) -> Result<(), anyhow::Error> {
        let mut store = self.inner.lock().await;

        // verify the domain is not already in use
        if store.iter().any(|e| e.domain == domain) {
            return Err(anyhow::anyhow!("domain already in use"));
        }

        let entry = StoreEntry { port, domain, path };

        store.push(entry.clone());

        Ok(())
    }

    /// Return the possible StoreEntry for a given domain, if any
    /// Example: api.localhost -> StoreEntry { port: , domain: "api.localhost", path: "" }
    pub async fn reverse_proxy(&self, host: String) -> Option<StoreEntry> {
        let store = self.inner.lock().await;
        store.iter().find(|e| e.domain == host).cloned()
    }

    /// When we start the daemon we set its process id
    pub fn set_daemon_process_id(&self, process_id: u32) -> Result<(), anyhow::Error> {
        let path = Path::new(&self.root_folder).join("daemon.pid");
        fs::write(path, process_id.to_string()).unwrap();
        Ok(())
    }

    /// When we stop the daemon we remove its process id
    pub fn remove_daemon_process_id(&self) -> Result<(), anyhow::Error> {
        let path = Path::new(&self.root_folder).join("daemon.pid");
        fs::remove_file(path).unwrap();
        Ok(())
    }

    pub fn get_daemon_process_id(&self) -> Result<u32, anyhow::Error> {
        let path = Path::new(&self.root_folder).join("daemon.pid");
        let content = fs::read_to_string(path)?;
        let process_id: u32 = content.parse()?;
        Ok(process_id)
    }
}
