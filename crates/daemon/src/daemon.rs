use std::io;

use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use paris::{error, info};
use tokio::net::TcpListener;

use noport_lib::store::Store;

use crate::{server::handle_request, watcher::run_watcher};

type ServerBuilder = hyper::server::conn::http1::Builder;

const DEFAULT_ADDR: &str = "127.0.0.1:2828";

pub async fn start_deamon(store: Store, addr: Option<String>) -> io::Result<()> {
    let addr = addr.unwrap_or_else(|| DEFAULT_ADDR.to_string());

    let host_folder = store.root_folder().join("hosts");

    let watcher_store_clone = store.clone();

    // run the watch process
    tokio::spawn(async move {
        info!("Running the hosts watching process");
        run_watcher(host_folder, watcher_store_clone).await.unwrap();
    });

    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        let store_clone = store.clone();

        tokio::spawn(async move {
            if let Err(e) = ServerBuilder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(
                    io,
                    service_fn(|req| handle_request(req, store_clone.clone())),
                )
                .with_upgrades()
                .await
            {
                error!("Error while handling the request: {}", e);
            }
        });
    }
}
