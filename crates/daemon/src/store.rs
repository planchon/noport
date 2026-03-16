use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

/// Add a domain to the database
/// The primary key is the path of the app, port and git status
pub async fn add_domain(
    store: Arc<Mutex<HashMap<String, u16>>>,
    path: String,
    domain: String,
    port: u16,
) -> Result<(), anyhow::Error> {
    let mut store = store.lock().await;

    let key = format!("{}-{}-{}", path, domain, port);
    store.insert(key, port);

    Ok(())
}

pub async fn get_domain(
    store: Arc<Mutex<HashMap<String, u16>>>,
    path: String,
    domain: String,
    port: u16,
) -> Option<u16> {
    let store = store.lock().await;
    let key = format!("{}-{}-{}", path, domain, port);
    store.get(&key).copied()
}
