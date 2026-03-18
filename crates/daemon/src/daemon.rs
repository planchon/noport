use std::io;

use tokio::net::TcpListener;

use noport_lib::store::Store;

use crate::server;

pub async fn start_deamon(store: Store, addr: Option<String>) -> io::Result<()> {
    let addr = addr.unwrap_or_else(|| "127.0.0.1:2828".to_string());

    let server = TcpListener::bind(&addr).await?;

    loop {
        let (stream, _) = server.accept().await?;

        let store_clone = store.clone();

        tokio::spawn(async move {
            let res = server::handle_request(stream, store_clone).await;
            if let Err(e) = res {
                println!("Error while handling request: {}", e);
            }
        });
    }
}
