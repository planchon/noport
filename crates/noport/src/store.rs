use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Store {
    inner: Arc<Mutex<HashMap<String, u16>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_domain(
        &self,
        path: String,
        domain: String,
        port: u16,
    ) -> Result<(), anyhow::Error> {
        let mut store = self.inner.lock().await;

        let key = format!("{}-{}-{}", path, domain, port);
        store.insert(key, port);

        Ok(())
    }

    pub async fn get_domain(&self, path: String, domain: String, port: u16) -> Option<u16> {
        let store = self.inner.lock().await;
        let key = format!("{}-{}-{}", path, domain, port);
        store.get(&key).copied()
    }
}
