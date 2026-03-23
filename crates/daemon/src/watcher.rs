use noport_lib::store::{Store, StoreEntry};
use notify::{Event, Watcher};
use paris::{error, info, success, warn};
use std::{
    fs::{self},
    path::PathBuf,
    sync::mpsc,
};

/// Watch over the hosts folder
/// One file is one process running
pub async fn run_watcher(hosts_folder: PathBuf, store: Store) -> notify::Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    // read all the files in the hosts folder
    first_scan(&hosts_folder, &store).await;

    let mut watcher = notify::recommended_watcher(tx)?;

    watcher.watch(hosts_folder.as_path(), notify::RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => match event.kind {
                notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                    info!(
                        "need rescan of the entry {}",
                        event.paths[0].to_str().unwrap()
                    );

                    for path in event.paths {
                        scan_one_file(path, &store).await;
                    }
                }
                notify::EventKind::Remove(_) => {
                    info!("can delete the entry {}", event.paths[0].to_str().unwrap());

                    for path in event.paths {
                        delete_one_file(path, &store).await;
                    }
                }
                _ => {}
            },
            Err(e) => {
                error!("got watch error {}", e);
            }
        }
    }

    Ok(())
}

async fn first_scan(path: &PathBuf, store: &Store) -> Result<(), anyhow::Error> {
    let paths = fs::read_dir(path).unwrap();

    for path in paths {
        let file = path.unwrap().path();
        info!("initial scan - scanning file {:?}", file);

        scan_one_file(file, &store).await;
    }

    Ok(())
}

async fn scan_one_file(path: PathBuf, store: &Store) -> Result<(), anyhow::Error> {
    let file_content = fs::read_to_string(path)?;

    let entry: StoreEntry = serde_json::from_str(&file_content)?;

    let mut store_inner = store.inner.lock().await;
    store_inner.push(entry.clone());

    success!("added on entry to the proxy {:?}", entry);

    Ok(())
}

async fn delete_one_file(path: PathBuf, store: &Store) -> Result<(), anyhow::Error> {
    let file_content = fs::read_to_string(path)?;

    let entry: StoreEntry = serde_json::from_str(&file_content)?;

    let mut store_inner = store.inner.lock().await;

    if let Some(idx) = store_inner.iter().position(|f| f.domain == entry.domain) {
        store_inner.remove(idx);
        success!("removed an entry from the store {:?}", entry);
    } else {
        warn!("trying to remove an element which is not stored");
    }

    Ok(())
}
