use super::{
    openssl::{generate_csr, generate_private_key, sign_certificate},
    types::CertificateConfig,
};
use crate::utils::logging::Logger;
use std::{fs, io, path::Path, process::Command};

pub fn generate_root_ca(config: &CertificateConfig, logger: &mut dyn Logger) -> io::Result<()> {
    logger.log("Generating root CA certificate...");

    fs::create_dir_all("certs/root-ca/")?;
    // Initialize CA directory
    fs::write("certs/root-ca/index.txt", "")?;
    let serial = Command::new("openssl")
        .args(&["rand", "-hex", "16"])
        .output()?;
    fs::write("certs/root-ca/serial", serial.stdout)?;

    // Check if certificate files already exist
    if Path::new("certs/root-ca/ca.crt").exists() && Path::new("certs/root-ca/ca.key").exists() {
        logger.log("Certificate files already exist. Skipping creation.");
        return Ok(());
    }

    // Create output directory
    fs::create_dir_all(&config.output_dir)?;
    let cert_path = config.output_dir.join("root-ca");
    fs::create_dir_all(&cert_path)?;

    // Generate private key
    let key_path = cert_path.join("ca.key");
    generate_private_key(key_path.to_str().unwrap(), config.key_size, logger)?;

    // Generate CA certificate
    let cert_file = cert_path.join("ca.crt");
    let output = std::process::Command::new("openssl")
        .args(&[
            "req",
            "-x509",
            "-new",
            "-nodes",
            "-key",
            key_path.to_str().unwrap(),
            "-days",
            &config.validity_days.to_string(),
            "-out",
            cert_file.to_str().unwrap(),
            "-subj",
            &format!("/CN={}", config.common_name),
        ])
        .output()?;

    if !output.status.success() {
        logger.log("Failed to generate root CA certificate");
        logger.debug_log(&format!("OpenSSL output: {:?}", output));
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to generate CA certificate",
        ));
    }

    logger.log("Root CA certificate generated successfully");
    Ok(())
}

pub fn generate_kubernetes_ca(
    config: &CertificateConfig,
    root_ca_path: &str,
    logger: &mut dyn Logger,
) -> io::Result<()> {
    logger.log("Generating Kubernetes CA certificate...");

    // Create output directory
    let cert_path = config.output_dir.join("kubernetes-ca");
    fs::create_dir_all(&cert_path)?;

    // Generate private key
    let key_path = cert_path.join("ca.key");
    generate_private_key(key_path.to_str().unwrap(), config.key_size, logger)?;

    // Generate CSR
    let csr_path = cert_path.join("ca.csr");
    generate_csr(
        config,
        key_path.to_str().unwrap(),
        csr_path.to_str().unwrap(),
        logger,
    )?;

    // Sign with root CA
    let cert_file = cert_path.join("ca.crt");
    sign_certificate(
        csr_path.to_str().unwrap(),
        cert_file.to_str().unwrap(),
        root_ca_path,
        &format!("{}/root-ca/ca.key", config.output_dir.display()),
        config,
        logger,
    )?;

    logger.log("Kubernetes CA certificate generated successfully");
    Ok(())
}
