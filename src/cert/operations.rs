// src/cert/operations.rs

use crate::utils::logging::Logger;
use std::path::Path;
use std::process::Command;
use std::{fs, io, path::PathBuf};

use super::openssl::{generate_csr, generate_private_key, sign_certificate};
use super::{CertificateConfig, CertificateType};

#[derive(Debug)]
pub enum CertOperationError {
    IoError(io::Error),
    CertGeneration(String),
    Distribution(String),
    Verification(String),
}

impl From<CertOperationError> for io::Error {
    fn from(error: CertOperationError) -> Self {
        match error {
            CertOperationError::IoError(e) => {
                io::Error::new(e.kind(), format!("Certificate operation IO error: {}", e))
            }
            CertOperationError::CertGeneration(s) => io::Error::new(
                io::ErrorKind::Other,
                format!("Certificate generation error: {}", s),
            ),
            CertOperationError::Distribution(s) => io::Error::new(
                io::ErrorKind::Other,
                format!("Certificate distribution error: {}", s),
            ),
            CertOperationError::Verification(s) => io::Error::new(
                io::ErrorKind::Other,
                format!("Certificate verification error: {}", s),
            ),
        }
    }
}

impl std::fmt::Display for CertOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO Error: {}", e),
            Self::CertGeneration(s) => write!(f, "Certificate Generation Error: {}", s),
            Self::Distribution(s) => write!(f, "Distribution Error: {}", s),
            Self::Verification(s) => write!(f, "Verification Error: {}", s),
        }
    }
}

impl std::error::Error for CertOperationError {}

impl From<io::Error> for CertOperationError {
    fn from(error: io::Error) -> Self {
        CertOperationError::IoError(error)
    }
}

pub struct CertificateOperations {
    logger: Box<dyn Logger>,
    remote_dir: String,
    remote_user: String,
    ssh_key_path: String,
}

impl CertificateOperations {
    pub fn new(
        logger: Box<dyn Logger>,
        remote_dir: String,
        remote_user: String,
        ssh_key_path: String,
    ) -> Self {
        Self {
            logger,
            remote_dir,
            remote_user,
            ssh_key_path,
        }
    }

    // Add public logging methods
    pub fn log(&mut self, message: &str) {
        self.logger.log(message);
    }

    pub fn debug_log(&mut self, message: &str) {
        self.logger.debug_log(message);
    }

    fn ensure_remote_directory(&mut self, host: &str) -> io::Result<()> {
        self.debug_log(&format!("Ensuring remote directory exists on {}", host));

        let ssh_output = Command::new("ssh")
            .args(&[
                "-i",
                &self.ssh_key_path,
                &format!("{}@{}", self.remote_user, host),
                &format!("sudo mkdir -p {}", self.remote_dir),
            ])
            .output()?;

        if !ssh_output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to create remote directory: {}",
                    String::from_utf8_lossy(&ssh_output.stderr)
                ),
            ));
        }

        self.log(&format!(
            "Remote directory {} created on {}",
            self.remote_dir, host
        ));
        Ok(())
    }

    // pub fn prepare_remote_hosts(&mut self, hosts: &[&str]) -> io::Result<()> {
    //     self.log("Preparing remote hosts...");
    //
    //     for host in hosts {
    //         if let Err(e) = self.ensure_remote_directory(host) {
    //             self.log(&format!("Failed to prepare remote host {}: {}", host, e));
    //             return Err(e);
    //         }
    //     }
    //
    //     self.log("All remote hosts prepared successfully");
    //     Ok(())
    // }

    pub fn generate_cert(
        &mut self,
        cert_name: &str,
        ca_dir: &str,
        config: &CertificateConfig,
        hosts: &[&str],
    ) -> Result<(), CertOperationError> {
        self.logger
            .log(&format!("Generating certificate for {}", cert_name));

        // Ensure all paths exist
        // let cert_dir = format!("certs/{}", cert_name);
        let cert_dir = config.output_dir.to_str().ok_or_else(|| {
            CertOperationError::CertGeneration("Invalid path for certificate directory".to_string())
        })?;

        match fs::create_dir_all(cert_dir) {
            Ok(_) => self.logger.log(&format!("Created directory: {}", cert_dir)),
            Err(e) => {
                self.logger
                    .log(&format!("Failed to create directory {}: {}", cert_dir, e));
                return Err(CertOperationError::IoError(e));
            }
        }

        // Set up paths
        let key_path = format!("{}/{}.key", cert_dir, cert_name);
        let csr_path = format!("{}/csr", cert_dir);
        let cert_path = format!("{}/{}.crt", cert_dir, cert_name);

        self.logger.debug_log(&format!(
            "cert_type {:?} for {}",
            config.cert_type, cert_name
        ));

        // For root CA, use its own directory for CA files since it's self-signed
        let (ca_cert, ca_key) = if config.cert_type == CertificateType::RootCA {
            (key_path.clone(), key_path.clone())
        } else {
            (format!("{}/ca.crt", ca_dir), format!("{}/ca.key", ca_dir))
        };

        self.logger.log("Generating private key");
        if let Err(e) = generate_private_key(&key_path, config.key_size, self.logger.as_mut()) {
            self.logger
                .log(&format!("Failed to generate private key: {}", e));
            return Err(CertOperationError::from(e));
        }

        self.logger.log("Generating CSR");
        if let Err(e) = generate_csr(config, &key_path, &csr_path, self.logger.as_mut()) {
            self.logger.log(&format!("Failed to generate CSR: {}", e));
            return Err(CertOperationError::from(e));
        }

        self.logger.log("Signing certificate");
        self.logger.debug_log(&format!(
            "cert_path:{}, ca_cert:{}, ca_key:{}",
            cert_path, ca_cert, ca_key
        ));
        if let Err(e) = sign_certificate(
            &csr_path,
            &cert_path,
            &ca_cert,
            &ca_key,
            config,
            self.logger.as_mut(),
        ) {
            self.logger
                .log(&format!("Failed to sign certificate: {}", e));
            return Err(CertOperationError::from(e));
        }

        Ok(())
    }

    // New method to set up all CA certificates
    pub fn setup_ca_certificates(&mut self, hosts: &[&str]) -> Result<(), CertOperationError> {
        // 1. Generate Root CA
        let root_config = CertificateConfig {
            cert_type: CertificateType::RootCA,
            common_name: "Kubernetes Root CA".to_string(),
            organization: Some("Kubernetes".to_string()),
            validity_days: 3650,
            key_size: 2048,
            output_dir: PathBuf::from("certs/root-ca"),
            alt_names: vec![],
            key_usage: vec![
                "critical".to_string(),
                "keyCertSign".to_string(),
                "cRLSign".to_string(),
            ],
            extended_key_usage: vec![],
        };

        self.logger.log("Generating Root CA certificate");
        self.generate_cert("ca", "certs/root-ca", &root_config, hosts)?;

        // Verify root CA files exist
        if !Path::new("certs/root-ca/ca.crt").exists()
            || !Path::new("certs/root-ca/ca.key").exists()
        {
            return Err(CertOperationError::CertGeneration(
                "Root CA files not generated properly".to_string(),
            ));
        }

        // 2. Generate Kubernetes CA
        let k8s_config = CertificateConfig {
            cert_type: CertificateType::KubernetesCA,
            common_name: "kubernetes-ca".to_string(),
            organization: Some("Kubernetes".to_string()),
            validity_days: 3650,
            key_size: 2048,
            output_dir: PathBuf::from("certs/kubernetes-ca"),
            alt_names: vec![],
            key_usage: vec![
                "critical".to_string(),
                "keyCertSign".to_string(),
                "cRLSign".to_string(),
            ],
            extended_key_usage: vec![],
        };

        self.logger.log("Generating Kubernetes CA certificate");
        self.generate_cert("ca", "certs/root-ca", &k8s_config, hosts)?;

        // 3. Create the CA chain
        if let Err(e) = self.create_ca_chain() {
            self.logger.log(&format!("Error {}", e))
        }

        Ok(())
    }

    fn create_ca_chain(&mut self) -> Result<(), CertOperationError> {
        self.logger.log("Creating CA chain");

        let root_ca_path = "certs/root-ca/ca.crt";
        let k8s_ca_path = "certs/kubernetes-ca/ca.crt";
        let chain_path = "certs/kubernetes-ca/ca-chain.crt";

        // Read both certificates
        let root_ca = fs::read_to_string(root_ca_path).map_err(|e| {
            CertOperationError::CertGeneration(format!("Failed to read root CA: {}", e))
        })?;

        let k8s_ca = fs::read_to_string(k8s_ca_path).map_err(|e| {
            CertOperationError::CertGeneration(format!("Failed to read kubernetes CA: {}", e))
        })?;

        // Concatenate certificates
        let chain_content = format!("{}\n{}", root_ca, k8s_ca);

        // Write the chain file
        fs::write(chain_path, chain_content).map_err(|e| {
            CertOperationError::CertGeneration(format!("Failed to create CA chain: {}", e))
        })?;

        self.logger.log("CA chain created successfully");
        Ok(())
    }

    pub fn distribute_certificates(
        &mut self,
        cert_name: &str,
        hosts: &Vec<String>,
    ) -> Result<(), CertOperationError> {
        for host in hosts {
            // Ensure remote directory exists before copying
            if let Err(e) = self.ensure_remote_directory(host) {
                self.log(&format!(
                    "Failed to create remote directory on {}: {}",
                    host, e
                ));
                return Err(CertOperationError::Distribution(format!(
                    "Failed to create remote directory on {}: {}",
                    host, e
                )));
            }

            self.copy_to_k8s_paths(cert_name, host).map_err(|e| {
                CertOperationError::Distribution(format!("Failed to distribute to {}: {}", host, e))
            })?;
        }
        Ok(())
    }

    // Distribution methods stay mostly the same but with improved error handling
    pub fn copy_to_k8s_paths(&mut self, cert_name: &str, remote_host: &str) -> io::Result<()> {
        self.logger.log(&format!(
            "Copying {} certificates to {}",
            cert_name, remote_host
        ));

        let cert_base_path = format!("certs/{}", cert_name);

        match cert_name {
            "kubernetes-ca" => self.copy_ca_certs(&cert_base_path, remote_host),
            // "ca-chain" => self.copy_ca_certs(&cert_base_path, remote_host),
            "kube-apiserver" | "controller-manager" | "scheduler" => {
                self.copy_component_certs(cert_name, &cert_base_path, remote_host)
            }
            "service-account" => self.copy_service_account_keys(&cert_base_path, remote_host),
            name if name.starts_with("node-") => {
                self.copy_node_certs(name, &cert_base_path, remote_host)
            }

            name if name.starts_with("kubeconfig/") => {
                let config_name = name.strip_prefix("kubeconfig/").unwrap();
                self.copy_with_sudo(
                    name,
                    &format!("/etc/kubernetes/{}", config_name),
                    remote_host,
                )
            }
            "encryption-config" => self.copy_with_sudo(
                "encryption-config.yaml",
                &format!("{}/encryption-config.yaml", self.remote_dir),
                remote_host,
            ),
            "root-ca" | "ca-chain" => Ok(()),
            _ => {
                self.logger
                    .log(&format!("Unknown file type {}, skipping copy.", cert_name));
                Ok(())
            }
        }
    }

    // Helper methods to make copy_to_k8s_paths cleaner
    fn copy_ca_certs(&mut self, base_path: &str, remote_host: &str) -> io::Result<()> {
        self.copy_with_sudo(
            &format!("{}/ca-chain.crt", base_path),
            &format!("{}/ca-chain.crt", self.remote_dir),
            remote_host,
        )?;
        self.copy_with_sudo(
            &format!("{}/ca.key", base_path),
            &format!("{}/kubernetes-ca.key", self.remote_dir),
            remote_host,
        )?;
        self.copy_with_sudo(
            &format!("{}/ca.crt", base_path),
            &format!("{}/kubernetes-ca.crt", self.remote_dir),
            remote_host,
        )
    }

    fn copy_component_certs(
        &mut self,
        component: &str,
        base_path: &str,
        remote_host: &str,
    ) -> io::Result<()> {
        self.copy_with_sudo(
            &format!("{}/{}.crt", base_path, component),
            &format!("{}/{}.crt", self.remote_dir, component),
            remote_host,
        )?;
        self.copy_with_sudo(
            &format!("{}/{}.key", base_path, component),
            &format!("{}/{}.key", self.remote_dir, component),
            remote_host,
        )
    }

    fn copy_service_account_keys(&mut self, base_path: &str, remote_host: &str) -> io::Result<()> {
        self.copy_with_sudo(
            &format!("{}/sa.key", base_path),
            &format!("{}/sa.key", self.remote_dir),
            remote_host,
        )?;
        self.copy_with_sudo(
            &format!("{}/sa.pub", base_path),
            &format!("{}/sa.pub", self.remote_dir),
            remote_host,
        )
    }

    fn copy_node_certs(
        &mut self,
        node_name: &str,
        base_path: &str,
        remote_host: &str,
    ) -> io::Result<()> {
        self.copy_with_sudo(
            &format!("{}/{}.crt", base_path, node_name),
            &format!("{}/{}.crt", self.remote_dir, node_name),
            remote_host,
        )?;
        self.copy_with_sudo(
            &format!("{}/{}.key", base_path, node_name),
            &format!("{}/{}.key", self.remote_dir, node_name),
            remote_host,
        )
    }

    pub fn copy_with_sudo(&mut self, source: &str, target: &str, host: &str) -> io::Result<()> {
        let temp_file = format!(
            "/tmp/{}",
            std::path::Path::new(source)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );

        // First, copy to temporary location
        let scp_output = Command::new("scp")
            .args(&[
                "-i",
                &self.ssh_key_path,
                source,
                &format!("{}@{}:{}", self.remote_user, host, temp_file),
            ])
            .output()?;

        if !scp_output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to copy file: {}",
                    String::from_utf8_lossy(&scp_output.stderr)
                ),
            ));
        }

        // Then move to final location with sudo
        let ssh_commands = format!(
            "sudo mv {} {} && sudo chown root:root {} && sudo chmod 644 {}",
            temp_file, target, target, target
        );

        let ssh_output = Command::new("ssh")
            .args(&[
                "-i",
                &self.ssh_key_path,
                &format!("{}@{}", self.remote_user, host),
                &ssh_commands,
            ])
            .output()?;

        if !ssh_output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to execute sudo commands: {}",
                    String::from_utf8_lossy(&ssh_output.stderr)
                ),
            ));
        }

        Ok(())
    }

    pub fn generate_service_account_keys(&mut self, hosts: &[&str]) -> io::Result<()> {
        self.logger.log("Generating service account keys");

        let sa_dir = PathBuf::from("certs/service-account");
        fs::create_dir_all(&sa_dir)?;

        // Generate private key
        let key_path = sa_dir.join("sa.key");
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
                "Failed to generate service account private key",
            ));
        }

        // Generate public key
        let pub_path = sa_dir.join("sa.pub");
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
                "Failed to generate service account public key",
            ));
        }

        // Distribute keys
        for host in hosts {
            self.copy_to_k8s_paths("service-account", host)?;
        }

        Ok(())
    }

    pub fn copy_kubeconfig(&mut self, name: &str, host: &str) -> io::Result<()> {
        self.copy_to_k8s_paths(&format!("kubeconfig/{}.conf", name), host)
    }

    pub fn copy_encryption_config(&mut self, host: &str) -> io::Result<()> {
        self.copy_to_k8s_paths("encryption-config.yaml", host)
    }
}
