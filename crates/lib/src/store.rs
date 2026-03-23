use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use paris::info;
use paris::success;
use paris::warn;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEntry {
    pub port: i32,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub inner: Arc<Mutex<Vec<StoreEntry>>>,

    host_folder: PathBuf,
    root_folder: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let home_folder = Path::new("/tmp/.noport").to_path_buf();

        let host_folder = home_folder.join("hosts");

        if !fs::exists(&home_folder).unwrap() {
            info!(
                "Creating the .noport folder ({})",
                home_folder.to_string_lossy()
            );
            fs::create_dir(&home_folder).unwrap();
        }
        if !fs::exists(&host_folder).unwrap() {
            fs::create_dir(&host_folder).unwrap();
        }

        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            root_folder: home_folder,
            host_folder: host_folder,
        }
    }

    pub fn root_folder(&self) -> PathBuf {
        self.root_folder.clone()
    }

    /// Set the global TLD
    pub fn set_tld(&self, tld: String) -> Result<(), anyhow::Error> {
        let path = Path::new(&self.root_folder).join("tld");
        fs::write(path, tld).unwrap();
        Ok(())
    }

    /// Get the global TLD
    pub fn get_tld(&self) -> String {
        let path = Path::new(&self.root_folder).join("tld");
        let content = fs::read_to_string(path).unwrap();
        content
    }

    /// CLI land
    /// Add a new proxy entry to the process
    pub async fn add_proxy_entry(
        &self,
        path: String,
        domain: String,
        port: i32,
    ) -> Result<(), anyhow::Error> {
        let entry = StoreEntry {
            port,
            domain: domain.clone(),
            path: path.clone(),
        };

        let host_file = format!("{}/{}", self.host_folder.to_string_lossy(), domain);
        let content = json!(entry).to_string();

        fs::write(host_file, content).unwrap();

        Ok(())
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
