use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use paris::info;
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
    inner: Arc<Mutex<Vec<StoreEntry>>>,

    host_folder: PathBuf,
    root_folder: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let home_folder = Path::new("/tmp/.noport").to_path_buf();

        let host_folder = home_folder.join("hosts");

        if !fs::exists(home_folder.clone()).unwrap() {
            info!(
                "Creating the .noport folder ({})",
                home_folder.to_string_lossy()
            );
        }
        if !fs::exists(host_folder.clone()).unwrap() {
            info!(
                "Creating the hosts folder ({})",
                host_folder.to_string_lossy()
            );
            fs::create_dir(host_folder.clone()).unwrap();
        }

        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            root_folder: home_folder,
            host_folder: host_folder,
        }
    }

    pub fn set_tld(&self, tld: String) -> Result<(), anyhow::Error> {
        let path = Path::new(&self.root_folder).join("tld");
        fs::write(path, tld).unwrap();
        Ok(())
    }

    pub fn get_tld(&self) -> String {
        let path = Path::new(&self.root_folder).join("tld");
        let content = fs::read_to_string(path).unwrap();
        content
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

        let entry = StoreEntry {
            port,
            domain: domain.clone(),
            path: path.clone(),
        };

        store.push(entry.clone());

        let host_file = format!("{}/{}", self.host_folder.to_string_lossy(), domain);
        let content = json!(entry).to_string();

        fs::write(host_file, content).unwrap();

        Ok(())
    }

    /// Return the possible StoreEntry for a given domain, if any
    /// Example: api.localhost -> StoreEntry { port: , domain: "api.localhost", path: "" }
    pub async fn reverse_proxy(&self, host: String) -> Option<StoreEntry> {
        let is_dev = env::var("DEV").is_ok();
        let mut store = self.inner.lock().await;
        let sub_domain = host.replace(".localhost", "");

        // in dev we also look for the file disk
        if !is_dev {
            if let Some(entry) = store.iter().find(|e| e.domain == sub_domain) {
                return Some(entry.clone());
            }
        }

        let host_file = format!("{}/{}", self.host_folder.to_string_lossy(), sub_domain);

        if fs::exists(host_file.clone()).unwrap() {
            let content = fs::read_to_string(host_file.clone()).unwrap();
            let entry: StoreEntry = serde_json::from_str(content.as_str()).unwrap();

            store.push(entry.clone());

            return Some(entry);
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
