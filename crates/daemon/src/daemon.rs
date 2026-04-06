use std::{fmt::Debug, fs::exists, sync::Arc};

use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use rustls::{
    ServerConfig,
    crypto::aws_lc_rs::sign::any_supported_type,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    server::ResolvesServerCert,
    sign::CertifiedKey,
};
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tracing::{error, info};

use noport_lib::{
    certs::{LocalCertificateAuthority, openssl::OpensslCerts},
    linux::get_home,
    machines::Machine,
    store::{NoPortStore, Store},
};
use tokio_rustls::TlsAcceptor;

use crate::{server::handle_request, socket::create_socket};

type ServerBuilder = hyper::server::conn::http1::Builder;

#[derive(Debug)]
struct NoPortCertResolver<C: LocalCertificateAuthority> {
    certs: C,
}

impl<C> NoPortCertResolver<C>
where
    C: LocalCertificateAuthority,
{
    pub fn new(certs: C) -> Self {
        Self { certs }
    }
}

impl<C> ResolvesServerCert for NoPortCertResolver<C>
where
    C: LocalCertificateAuthority + Send + Sync + Debug,
{
    fn resolve(
        &self,
        client_hello: rustls::server::ClientHello<'_>,
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        match client_hello.server_name() {
            Some(sni) => {
                let cert = self.certs.get_certificate(sni.to_string());

                if cert.is_err() {
                    error!("could not find the certificate for the host {}", sni);
                    return None;
                }

                let host_cert = cert.unwrap();

                let cert = CertificateDer::from_pem_file(host_cert.certificate).unwrap();
                let private_key = PrivateKeyDer::from_pem_file(host_cert.secret).unwrap();

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

pub async fn start_deamon<M: Machine + Clone + Debug + Send + Sync>(
    store: Store<M>,
    addr: String,
    https: bool,
    shutdown_tx: Sender<()>,
) -> Result<(), anyhow::Error> {
    let socket_store = store.clone();

    let certs = OpensslCerts::new(store.get_machine());

    // run the socket (interaction between CLI and Daemon)
    tokio::spawn(async move {
        if let Err(e) = create_socket(&socket_store, shutdown_tx).await {
            error!("error while creating the socket (path={})", e);
        }
    });

    let listener = TcpListener::bind(&addr).await?;

    info!("Starting the reverse proxy (addr={})", addr);

    let tls_config = match https {
        false => None,
        true => Some(Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_cert_resolver(Arc::new(NoPortCertResolver { certs })),
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
