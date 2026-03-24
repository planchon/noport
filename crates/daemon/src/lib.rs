use noport_lib::{
    hosts::write_host,
    store::{Store, StoreEntry},
};
use paris::{success, warn};

pub mod daemon;
mod server;
mod socket;

pub async fn handle_one_entry(entry: StoreEntry, store: &Store) -> Result<(), anyhow::Error> {
    let mut store_inner = store.inner.lock().await;

    let host = entry.domain.clone();
    if let Err(e) = write_host(host) {
        warn!("could not add the host to the host file {}", e);
    }

    // add to the entry to the store
    store_inner.push(entry.clone());

    success!("added on entry to the proxy {:?}", entry);

    Ok(())
}
