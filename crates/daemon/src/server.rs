use std::io;

use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{Method, Response, StatusCode, Uri, body::Bytes, upgrade::Upgraded};

use ansi_term::Colour;
use hyper_util::rt::TokioIo;
use noport_lib::store::Store;
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

fn host_addr(uri: &Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

async fn tunnel(upgraded: Upgraded, addr: String) -> io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    println!(
        "client wrote {}b and received {}b",
        from_client, from_server
    );

    Ok(())
}

pub async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    store: Store,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error> {
    let host = extract_host(&req);

    if host.is_none() {
        println!("{}", Colour::Red.paint("cannot find the host name"));
        // THIS IS WRONG. we shoudl return an http error.
        return Err(anyhow::anyhow!("cannot find the host"));
    }

    let host_value = host.unwrap().clone();
    let store_entry = store.reverse_proxy(host_value.clone()).await;

    if store_entry.is_none() {
        println!(
            "{} {}",
            Colour::Red.paint("cannot find the store entry for host"),
            Colour::Fixed(27).paint(host_value)
        );
        // THIS IS WRONG. we shoudl return an http error.
        return Err(anyhow::anyhow!("cannot find the store entry"));
    }

    let port = store_entry.unwrap().port;
    let addr = format!("127.0.0.1:{}", port);
    let method = req.method();

    println!("trying to connect to {}, {}", addr, method);

    // bi directionnal tunnel for websocket and that stuffs
    if method == Method::CONNECT || req.headers().contains_key("Upgrade") {
        println!("Upgrading the connection");

        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, addr).await {
                        println!("server io error {}", e);
                    }
                }
                Err(e) => {
                    println!("upgrading error: {:?}", e);
                }
            }
        });

        return Ok(Response::new(empty()));
    }

    // everything else is a normal connection
    let stream = TcpStream::connect(("127.0.0.1", port as u16))
        .await
        .unwrap();
    let io = TokioIo::new(stream);

    let (mut sender, conn) = ClientBuilder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(io)
        .await?;

    tokio::task::spawn(async move {
        if let Err(e) = conn.await {
            println!("error while connecting {:?}", e);
        }
    });

    let resp = sender.send_request(req).await?;

    Ok(resp.map(|b| b.boxed()))
}
