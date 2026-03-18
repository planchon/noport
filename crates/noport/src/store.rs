use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::{collections::HashMap, env};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEntry {
    pub port: u16,
    pub domain: String,
}

#[derive(Debug, Clone)]
pub struct Store {
    inner: Arc<Mutex<HashMap<String, StoreEntry>>>,
    root_folder: String,
}

impl Store {
    pub fn new() -> Self {
        let home_dir = env::home_dir().unwrap();
        let home_folder = home_dir.join(".noport").to_string_lossy().to_string();

        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            root_folder: home_folder,
        }
    }

    pub fn get_root_folder(&self) -> String {
        self.root_folder.clone()
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

    fn read_entry_from_disk(&self, key: String) -> Option<StoreEntry> {
        let path = Path::new(&self.root_folder).join(key);
        if path.exists() {
            let content = fs::read_to_string(path).unwrap();
            let entry: StoreEntry = serde_json::from_str(&content).unwrap();
            return Some(entry);
        }
        None
    }

    fn write_entry_to_disk(&self, key: String, entry: StoreEntry) -> Result<(), anyhow::Error> {
        let path = Path::new(&self.root_folder).join(key);
        fs::write(path, serde_json::to_string(&entry).unwrap()).unwrap();
        Ok(())
    }

    pub async fn add_domain(
        &self,
        path: String,
        domain: String,
        port: u16,
    ) -> Result<(), anyhow::Error> {
        let mut store = self.inner.lock().await;

        let key = format!("{}-{}-{}", path, domain, port);
        let entry = StoreEntry { port, domain };

        store.insert(key.clone(), entry.clone());

        self.write_entry_to_disk(key, entry)?;

        Ok(())
    }

    pub async fn get_domain(&self, path: String, domain: String, port: u16) -> Option<StoreEntry> {
        let store = self.inner.lock().await;
        let key = format!("{}-{}-{}", path, domain, port);

        // check in memory first
        let entry = store.get(&key);

        if let Some(entry) = entry {
            return Some(entry.clone());
        }

        // check on the disk
        let entry = self.read_entry_from_disk(key);
        if let Some(entry) = entry {
            return Some(entry);
        }

        None
    }
}
