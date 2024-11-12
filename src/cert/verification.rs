// src/cert/verification.rs
use crate::utils::logging::Logger;
use std::{fs, io, path::PathBuf, process::Command};

pub struct CertificateVerifier {
    logger: Box<dyn Logger>,
    remote_user: String,
    remote_dir: String,
    ssh_key_path: String,
}

impl CertificateVerifier {
    pub fn new(
        logger: Box<dyn Logger>,
        remote_user: String,
        remote_dir: String,
        ssh_key_path: String,
    ) -> Self {
        Self {
            logger,
            remote_user,
            remote_dir,
            ssh_key_path,
        }
    }

    pub fn verify_remote_certificates(&mut self, hosts: &[String]) -> io::Result<()> {
        self.logger.log("Verifying certificates on remote hosts...");

        for host in hosts {
            self.logger.log(&format!("Verifying certificates on host {}...", host));

            // Set up temp directory
            let temp_dir = format!("/tmp/cert-verify-{}", host);
            fs::create_dir_all(&temp_dir)?;

            let result = self.verify_host_certificates(host, &temp_dir);

            // Cleanup
            let _ = fs::remove_dir_all(&temp_dir);

            if let Err(e) = result {
                self.logger.log(&format!(
                    "Failed to verify certificates on {}: {}",
                    host, e
                ));
            }
        }

        Ok(())
    }

    fn verify_host_certificates(&mut self, host: &str, temp_dir: &str) -> io::Result<()> {
        let certificates = [
            ("kubernetes-ca-chain.crt", None),
            ("kube-apiserver.crt", Some("kubernetes-ca-chain.crt")),
            ("controller-manager.crt", Some("kubernetes-ca-chain.crt")),
            ("scheduler.crt", Some("kubernetes-ca-chain.crt")),
        ];

        for (cert_name, ca_cert) in certificates {
            let remote_path = format!("{}/{}", self.remote_dir, cert_name);
            let local_path = format!("{}/{}", temp_dir, cert_name);

            if let Err(e) = self.copy_from_remote(host, &remote_path, &local_path) {
                self.logger.log(&format!(
                    "Failed to copy {} from {}: {}",
                    cert_name, host, e
                ));
                continue;
            }

            // Verify certificate
            let ca_path = ca_cert.map(|ca| format!("{}/{}", temp_dir, ca));
            self.verify_certificate(&local_path, ca_path.as_deref())?;
        }

        Ok(())
    }

    fn copy_from_remote(&self, host: &str, remote_path: &str, local_path: &str) -> io::Result<()> {
        let ssh_key_path = shellexpand::tilde(&self.ssh_key_path).to_string();

        let output = Command::new("scp")
            .args(&[
                "-i",
                &ssh_key_path,
                &format!("{}@{}:{}", self.remote_user, host, remote_path),
                local_path,
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to copy {} from {}", remote_path, host),
            ));
        }

        Ok(())
    }

    pub fn verify_certificate(&mut self, cert_path: &str, ca_cert: Option<&str>) -> io::Result<()> {
        // Basic certificate info check
        let basic_check = Command::new("openssl")
            .args(&["x509", "-in", cert_path, "-noout", "-text"])
            .output()?;

        if !basic_check.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Basic certificate check failed for {}", cert_path)
            ));
        }

        // Verify against CA if provided
        if let Some(ca) = ca_cert {
            let output = Command::new("openssl")
                .args(&["verify", "-CAfile", ca, cert_path])
                .output()?;

            if !output.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Certificate chain verification failed for {}", cert_path)
                ));
            }
        }

        Ok(())
    }

    pub fn verify_service_account_keypair(&mut self, sa_dir: &PathBuf) -> io::Result<()> {
        self.logger.log("Verifying service account key pair...");

        let output = Command::new("openssl")
            .args(&[
                "rsa",
                "-in",
                sa_dir.join("sa.key").to_str().unwrap(),
                "-check",
                "-noout",
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Service account key verification failed",
            ));
        }

        self.logger.log("Service account key pair verification successful");
        Ok(())
    }
}
