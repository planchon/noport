use anyhow::anyhow;
use hyper::{Response, StatusCode, upgrade::OnUpgrade};

use hyper_util::rt::TokioIo;
use noport_lib::store::Store;
use paris::{error, info};
use tokio::net::TcpStream;

type ClientBuilder = hyper::client::conn::http1::Builder;

fn extract_host(req: &hyper::Request<hyper::body::Incoming>) -> Option<String> {
    // http uri
    if let Some(h) = req.uri().host() {
        return Some(h.to_string());
    }

    // header
    let headers = req.headers();
    if let Some((_, header_value)) = headers.iter().find(|h| h.0 == "host") {
        let value = header_value.to_str().unwrap();

        match value.find(":") {
            None => {
                return Some(value.to_string());
            }
            Some(val) => return Some(value[..val].to_string()),
        }
    }

    None
}

// async fn tunnel(upgraded: Upgraded, addr: String) -> io::Result<()> {
//     let mut server = TcpStream::connect(addr).await?;
//     let mut upgraded = TokioIo::new(upgraded);

//     tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

//     Ok(())
// }

async fn tunnel(client: OnUpgrade, server: OnUpgrade) -> Result<(), Box<dyn std::error::Error>> {
    let (client_upgraded, server_upgraded) = tokio::try_join!(client, server)?;

    let mut client_io = TokioIo::new(client_upgraded);
    let mut server_io = TokioIo::new(server_upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut client_io, &mut server_io).await?;

    info!(
        "Tunnel closed, received {}b, sent {}b",
        from_client, from_server
    );

    Ok(())
}

pub async fn handle_request(
    mut req: hyper::Request<hyper::body::Incoming>,
    store: Store,
) -> Result<Response<hyper::body::Incoming>, anyhow::Error> {
    let host = extract_host(&req);

    if host.is_none() {
        error!("Cannot find the host name");

        // THIS IS WRONG. we shoudl return an http error.
        return Err(anyhow::anyhow!("cannot find the host"));
    }

    let host_value = host.unwrap().clone();
    let store_entry = store.reverse_proxy(host_value.clone()).await;

    if store_entry.is_none() {
        error!(
            "Cannot find the store entry for the host {} (have you launched noport before the command?)",
            host_value
        );
        // THIS IS WRONG. we shoudl return an http error.
        return Err(anyhow::anyhow!("cannot find the store entry"));
    }

    let mut client_upgrade = None;
    if req.headers().contains_key("Upgrade") {
        if let Some(on_upgrade) = req.extensions_mut().remove::<OnUpgrade>() {
            client_upgrade = Some(on_upgrade);
        }
    }

    let port = store_entry.unwrap().port;

    // everything else is a normal connection
    let stream = TcpStream::connect(("127.0.0.1", port as u16))
        .await
        .unwrap();
    let io = TokioIo::new(stream);

    // forward the request to the server
    let (mut sender, conn) = hyper::client::conn::http1::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(io)
        .await?;

    let conn = conn.with_upgrades();

    tokio::task::spawn(async move {
        if let Err(e) = conn.await {
            error!("Error while connecting {}", e);
        }
    });

    let resp = sender.send_request(req).await?;

    if resp.status() == StatusCode::SWITCHING_PROTOCOLS {
        if let Some(client_on_upgrade) = client_upgrade {
            let server_on_upgrade = resp.extensions().get::<OnUpgrade>().cloned().unwrap();

            tokio::spawn(async move {
                if let Err(e) = tunnel(client_on_upgrade, server_on_upgrade).await {
                    error!("Upgrade tunnel error {}", e);
                }
            });

            Ok(resp)
        } else {
            Err(anyhow!("should not be possible"))
        }
    } else {
        Ok(resp)
    }
}
