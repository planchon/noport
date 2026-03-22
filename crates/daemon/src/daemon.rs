use std::io;

use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use paris::error;
use tokio::net::TcpListener;

use noport_lib::store::Store;

use crate::server::handle_request;

type ServerBuilder = hyper::server::conn::http1::Builder;

const DEFAULT_ADDR: &str = "127.0.0.1:2828";

pub async fn start_deamon(store: Store, addr: Option<String>) -> io::Result<()> {
    let addr = addr.unwrap_or_else(|| DEFAULT_ADDR.to_string());

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
