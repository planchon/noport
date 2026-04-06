use std::path::{Path, PathBuf};

use anyhow::Result;

pub mod openssl;

#[derive(Debug, Clone)]
struct PrivateKey(PathBuf);

#[derive(Debug, Clone)]
struct Certificate {
    pub certificate: PathBuf,
    pub secret: PathBuf,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum CertificateErrors {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("error while generating the private key")]
    PrivateKeyGeneration,
    #[error("CA not found")]
    NoCa,
    #[error("openssl command failed")]
    OpensslCommandFailed,
}

pub trait LocalCertificateAuthority {
    /// Generate a private key using the best cryptographic algorithm available
    /// by default tries to use the `openssl` binary
    fn generate_private_key(path: &Path) -> impl Future<Output = Result<(), CertificateErrors>>;

    /// Generate a CA signing key and certificate.
    /// Saves them into `~/.noport/ca` folder
    fn generate_ca(&self) -> impl Future<Output = Result<Certificate, CertificateErrors>>;

    fn get_ca(&self) -> Result<Certificate, CertificateErrors>;

    /// Generate a host certificate and private key
    /// Saves them into `~/.noport/certs/host_name` folder
    fn generate_server_certificate(
        &self,
        server: String,
    ) -> impl Future<Output = Result<Certificate, CertificateErrors>>;
}
