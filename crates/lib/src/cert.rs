use std::{
    env::{self, home_dir},
    fmt::format,
    fs::{create_dir, exists, write},
    io,
    path::Path,
    process::{self, Command},
};

use anyhow::{Ok, anyhow};
use nix::unistd::Uid;

/// Setup all the certificates
pub fn setup_ca() -> Result<(), anyhow::Error> {
    let certs_folder = home_dir()
        .ok_or(anyhow!("home not found"))?
        .join(".noport/certs");
    if !exists(&certs_folder)? {
        create_dir(&certs_folder)?;
    }

    let ca_key_path = certs_folder.join("ca_key.pem");
    let ca_cert_path = certs_folder.join("ca_cert.pem");

    create_private_key(&ca_key_path)?;
    create_ca_cert(&ca_key_path, &ca_cert_path)?;

    Ok(())
}

pub fn trust_certificate() -> Result<process::ExitStatus, anyhow::Error> {
    if !Uid::current().is_root() {
        return Err(anyhow::Error::msg("THis command should be run as sudo"));
    }

    let ca_cert_path = home_dir()
        .ok_or(anyhow!("home not found"))?
        .join(".noport/certs/ca_cert.pem");

    match env::consts::OS {
        "macos" => Command::new("security")
            .args(vec![
                "add-trusted-cert",
                "-d",
                "-r",
                "trustRoot",
                "-k",
                "/Library/Keychains/System.keychain",
                ca_cert_path.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| anyhow::Error::new(e)),
        _ => return Err(anyhow!("OS not supported")),
    }
}

pub fn generate_certificate_for_host(host: &str) -> Result<(), anyhow::Error> {
    let home_path = home_dir().unwrap().join(".noport/certs");

    if !exists(&home_path)? {
        return Err(anyhow!(
            "The CA certificate do not exists (no .noport/certs folder)"
        ));
    }

    let ca_cert = home_path.join("ca_cert.pem");
    let ca_key = home_path.join("ca_key.pem");

    if !exists(&ca_cert)? || !exists(&ca_key)? {
        return Err(anyhow!(
            "The CA certificate do not exists (no ca_cert.pem / ca_key.pem file in .noport/certs)"
        ));
    }

    let cert_path = home_path.join(format!("{}_cert.pem", host).as_str());
    let key_path = home_path.join(format!("{}_key.pem", host).as_str());
    let csr_path = home_path.join(format!("{}.csr", host).as_str());
    let ext_file = home_path.join(format!("{}-ext.cnf", host).as_str());

    create_private_key(&key_path)?;
    create_csr(&key_path, &csr_path, host)?;
    generate_ext_file(&ext_file, host)?;
    sign_cert(&ca_key, &ca_cert, &csr_path, &ext_file, &cert_path)?;

    Ok(())
}

/// Generate the CA key
fn create_private_key(key_path: &Path) -> Result<process::ExitStatus, io::Error> {
    Command::new("openssl")
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
}

/// Generate the CA cert
fn create_ca_cert(
    ca_key_path: &Path,
    ca_cert_path: &Path,
) -> Result<process::ExitStatus, io::Error> {
    Command::new("openssl")
        .args(vec![
            "req",
            "-new",
            "-x509",
            "-sha256",
            "-key",
            ca_key_path.to_str().unwrap(),
            "-out",
            ca_cert_path.to_str().unwrap(),
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
}

/// Create the certificate server request
fn create_csr(
    server_key: &Path,
    csr_path: &Path,
    domain: &str,
) -> Result<process::ExitStatus, io::Error> {
    Command::new("openssl")
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
}

fn sign_cert(
    ca_key: &Path,
    ca_cert: &Path,
    csr: &Path,
    ext_file: &Path,
    domain_cert: &Path,
) -> Result<process::ExitStatus, io::Error> {
    Command::new("openssl")
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
}

fn generate_ext_file(path: &Path, host: &str) -> Result<(), io::Error> {
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
}
