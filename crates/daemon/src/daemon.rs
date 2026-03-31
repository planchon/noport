use std::{env::home_dir, fs::exists, sync::Arc};

use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use paris::{error, info, success};
use rustls::{
    ServerConfig,
    crypto::aws_lc_rs::sign::any_supported_type,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    server::ResolvesServerCert,
    sign::CertifiedKey,
};
use tokio::{net::TcpListener, sync::mpsc::Sender};

use noport_lib::{cert::generate_certificate_for_host, store::Store};
use tokio_rustls::TlsAcceptor;

use crate::{server::handle_request, socket::create_socket};

type ServerBuilder = hyper::server::conn::http1::Builder;

#[derive(Debug)]
struct NoPortCertResolver {}

impl ResolvesServerCert for NoPortCertResolver {
    fn resolve(
        &self,
        client_hello: rustls::server::ClientHello<'_>,
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        match client_hello.server_name() {
            Some(sni) => {
                let base_cert_folder = home_dir().unwrap().join(".noport/certs");

                let cert_file = base_cert_folder
                    .clone()
                    .join(format!("{}_cert.pem", sni).as_str());
                let key_file = base_cert_folder
                    .clone()
                    .join(format!("{}_key.pem", sni).as_str());

                // we generate on the fly the certificate
                if !exists(&cert_file).unwrap() {
                    info!("generating a certificate for the host {}", sni);
                    if let Err(e) = generate_certificate_for_host(sni) {
                        error!("Error while generating the certificate {}", e);
                        return None;
                    }
                }

                let cert = CertificateDer::from_pem_file(cert_file).unwrap();
                let private_key = PrivateKeyDer::from_pem_file(key_file).unwrap();

                Some(Arc::new(CertifiedKey {
                    cert: vec![cert],
                    key: any_supported_type(&private_key).unwrap(),
                    ocsp: None,
                }))
            }
            None => None,
        }
    }
}

pub async fn start_deamon(
    store: Store,
    addr: String,
    https: bool,
    shutdown_tx: Sender<()>,
) -> Result<(), anyhow::Error> {
    let socket_store = store.clone();

    // run the socket (interaction between CLI and Daemon)
    tokio::spawn(async move {
        if let Err(e) = create_socket(&socket_store, shutdown_tx).await {
            error!("error while creating the socket (path={})", e);
        }
    });

    let listener = TcpListener::bind(&addr).await?;

    success!("Starting the reverse proxy (addr={})", addr);

    let tls_config = match https {
        false => None,
        true => Some(Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_cert_resolver(Arc::new(NoPortCertResolver {})),
        )),
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let store_clone = store.clone();

        if https {
            let acceptator = TlsAcceptor::from(tls_config.clone().unwrap());
            let decoded_stream = acceptator.accept(stream).await.unwrap();

            let io = TokioIo::new(decoded_stream);

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
        } else {
            let io = TokioIo::new(stream);
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
}
