use std::io;

use tokio::net::TcpListener;

mod server;

pub async fn start_proxy(addr: Option<String>) -> io::Result<()> {
    let addr = addr.unwrap_or_else(|| "127.0.0.1:2828".to_string());

    let server = TcpListener::bind(&addr).await?;

    loop {
        let (stream, _) = server.accept().await?;

        tokio::spawn(async move {
            server::handle_request(stream).await.unwrap();
        });
    }
}
