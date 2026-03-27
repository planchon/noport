use std::{fs, io, process::exit};

use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use paris::{error, info, success};
use tokio::{
    net::TcpListener,
    sync::mpsc::{Receiver, Sender},
};

use noport_lib::{communication::get_socket, store::Store};

use crate::{server::handle_request, socket::create_socket};

type ServerBuilder = hyper::server::conn::http1::Builder;

pub async fn start_deamon(store: Store, addr: String, shutdown_tx: Sender<()>) -> io::Result<()> {
    let socket_store = store.clone();

    // run the socket (interaction between CLI and Daemon)
    tokio::spawn(async move {
        if let Err(e) = create_socket(&socket_store, shutdown_tx).await {
            error!("error while creating the socket (path={})", e);
        }
    });

    let listener = TcpListener::bind(&addr).await?;

    success!("Starting the reverse proxy (addr={})", addr);

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
