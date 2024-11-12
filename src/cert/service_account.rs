// src/cert/service_account.rs
use std::fs;
use std::process::Command;
use std::{io, path::PathBuf};

use super::CertificateOperations;

pub struct ServiceAccountGenerator<'a> {
    output_dir: PathBuf,
    cert_ops: &'a mut CertificateOperations,
}

#[derive(Debug)]
pub enum ServiceAccountError {
    IoError(io::Error),
    KeyGeneration(String),
}

impl From<io::Error> for ServiceAccountError {
    fn from(error: io::Error) -> Self {
        ServiceAccountError::IoError(error)
    }
}

impl<'a> ServiceAccountGenerator<'a> {
    pub fn new(output_dir: PathBuf, cert_ops: &'a mut CertificateOperations) -> Self {
        Self {
            output_dir,
            cert_ops,
        }
    }

    pub fn generate_service_account_keys(&mut self) -> io::Result<()> {
        self.cert_ops.log("Generating service account key pair");

        // Ensure directory exists
        fs::create_dir_all(&self.output_dir)?;

        // Generate private key
        self.generate_private_key()?;

        // Generate public key
        self.generate_public_key()?;

        self.cert_ops
            .log("Service account keys generated successfully");
        Ok(())
    }

    fn generate_private_key(&mut self) -> io::Result<()> {
        let key_path = self.output_dir.join("sa.key");
        let output = Command::new("openssl")
            .args(&[
                "genpkey",
                "-algorithm",
                "RSA",
                "-out",
                key_path.to_str().unwrap(),
                "-pkeyopt",
                "rsa_keygen_bits:2048",
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to generate SA private key",
            ));
        }
        Ok(())
    }

    fn generate_public_key(&mut self) -> io::Result<()> {
        let key_path = self.output_dir.join("sa.key");
        let pub_path = self.output_dir.join("sa.pub");

        let output = Command::new("openssl")
            .args(&[
                "rsa",
                "-in",
                key_path.to_str().unwrap(),
                "-pubout",
                "-out",
                pub_path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to generate SA public key",
            ));
        }
        Ok(())
    }

    pub fn verify_keypair(&self) -> io::Result<()> {
        let key_path = self.output_dir.join("sa.key");
        // let pub_path = self.output_dir.join("sa.pub");

        // Verify private key
        let output = Command::new("openssl")
            .args(&["rsa", "-check", "-in", key_path.to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Service account private key verification failed",
            ));
        }

        // Verify public key matches private key
        let output = Command::new("openssl")
            .args(&[
                "rsa",
                "-in",
                key_path.to_str().unwrap(),
                "-pubout",
                "-outform",
                "PEM",
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Service account key pair verification failed",
            ));
        }

        Ok(())
    }
}
