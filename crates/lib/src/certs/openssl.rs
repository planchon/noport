use std::path::Path;

use crate::{
    certs::{Certificate, CertificateErrors, LocalCertificateAuthority},
    machines::Machine,
};

use anyhow::Result;
use tokio::{
    fs::{create_dir, write},
    process::Command,
};

struct OpensslCerts<M> {
    local_machine: M,
}

impl<M> OpensslCerts<M> {
    async fn generate_csr(
        domain: &str,
        server_key: &Path,
        csr_path: &Path,
    ) -> Result<(), CertificateErrors> {
        let exit_status = Command::new("openssl")
            .args(vec![
                "req",
                "-new",
                "-key",
                server_key.to_str().unwrap(),
                "-out",
                csr_path.to_str().unwrap(),
                "-subj",
                format!("/CN={}", domain).as_str(),
            ])
            .status()
            .await?;

        if !exit_status.success() {
            return Err(CertificateErrors::OpensslCommandFailed);
        }

        Ok(())
    }

    async fn generate_ext_file(path: &Path, host: &str) -> Result<(), CertificateErrors> {
        write(
            path,
            format!(
                "
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage=digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:{},DNS:*.{}
            ",
                host, host
            ),
        )
        .await?;

        Ok(())
    }

    async fn sign_cert(
        ca_key: &Path,
        ca_cert: &Path,
        domain_cert: &Path,
        csr: &Path,
        ext_file: &Path,
    ) -> Result<(), CertificateErrors> {
        let exit = Command::new("openssl")
            .args(vec![
                "x509",
                "-req",
                "-sha256",
                "-in",
                csr.to_str().unwrap(),
                "-CA",
                ca_cert.to_str().unwrap(),
                "-CAkey",
                ca_key.to_str().unwrap(),
                "-CAcreateserial",
                "-out",
                domain_cert.to_str().unwrap(),
                "-days",
                "365",
                "-extfile",
                ext_file.to_str().unwrap(),
            ])
            .status()
            .await?;

        if !exit.success() {
            return Err(CertificateErrors::OpensslCommandFailed);
        }

        Ok(())
    }
}

impl<M> LocalCertificateAuthority for OpensslCerts<M>
where
    M: Machine,
{
    async fn generate_private_key(key_path: &Path) -> Result<(), CertificateErrors> {
        if let Ok(res) = Command::new("openssl")
            .args(vec![
                "ecparam",
                "-genkey",
                "-name",
                "prime256v1",
                "-noout",
                "-out",
                key_path.to_str().unwrap(),
            ])
            .status()
            .await
        {
            if !res.success() {
                return Err(CertificateErrors::PrivateKeyGeneration);
            }
        }

        Ok(())
    }

    async fn generate_ca(&self) -> Result<Certificate, CertificateErrors> {
        let ca_key = self.local_machine.get_ca_folder().join("ca_key.pem");
        let ca_cert = self.local_machine.get_ca_folder().join("ca_cert.pem");

        Self::generate_private_key(&ca_key).await?;

        Command::new("openssl")
            .args(vec![
                "req",
                "-new",
                "-x509",
                "-sha256",
                "-key",
                ca_key.to_str().unwrap(),
                "-out",
                ca_cert.to_str().unwrap(),
                "-days",
                "3650",
                "-subj",
                "/CN=noport Local CA",
                "-addext",
                "basicConstraints=critical,CA:TRUE",
                "-addext",
                "keyUsage=critical,keyCertSign,cRLSign",
            ])
            .status()
            .await?;

        Ok(Certificate {
            certificate: ca_cert,
            secret: ca_key,
        })
    }

    fn get_ca(&self) -> Result<Certificate, CertificateErrors> {
        let ca_cert = self.local_machine.get_ca_folder().join("ca_cert.pem");
        let ca_key = self.local_machine.get_ca_folder().join("ca_key.pem");

        if !ca_cert.exists() || !ca_key.exists() {
            return Err(CertificateErrors::NoCa);
        }

        Ok(Certificate {
            certificate: ca_cert,
            secret: ca_key,
        })
    }

    async fn generate_server_certificate(
        &self,
        host: String,
    ) -> Result<Certificate, CertificateErrors> {
        let ca = self.get_ca()?;
        let server_path = self.local_machine.get_certs_folder().join(&host);

        if !server_path.exists() {
            create_dir(&server_path).await?;
        }

        let cert_file = server_path.join("cert.pem");
        let secret = server_path.join("key.pem");

        let csr_path = server_path.join(format!("{}.csr", host.clone()).as_str());
        let ext_path = server_path.join(format!("{}-ext.cnf", host.clone()).as_str());

        Self::generate_private_key(&secret).await?;
        Self::generate_csr(&host, &secret, &csr_path).await?;
        Self::generate_ext_file(&ext_path, &host).await?;

        Ok(Certificate {
            certificate: cert_file,
            secret,
        })
    }
}
