use std::io;

use tokio::net::TcpListener;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

mod server;
mod store;

pub async fn start_deamon(addr: Option<String>) -> io::Result<()> {
    let store = Arc::new(Mutex::new(HashMap::new()));

    let addr = addr.unwrap_or_else(|| "127.0.0.1:2828".to_string());

    let server = TcpListener::bind(&addr).await?;

    loop {
        let (stream, _) = server.accept().await?;

        let storeClone = store.clone();

        tokio::spawn(async move {
            server::handle_request(stream, storeClone).await;
        });
    }
}
