use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tempdir::TempDir;
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
        let tmp_folder = TempDir::new("noport").expect("Failed to create temp directory");
        let root_folder = tmp_folder.path().to_string_lossy().to_string();

        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            root_folder,
        }
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
