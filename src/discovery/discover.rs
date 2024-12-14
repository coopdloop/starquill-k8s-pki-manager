use crate::app::{CertManager, CertStatus}; // Assuming CertStatus is in types module
use crate::discovery::kubeconfig::{ClusterConfig, ContextConfig, KubeConfig, UserConfig};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::{fs, io};
use tokio::sync::RwLock;
use utoipa::ToSchema;
use x509_parser::prelude::{FromDer, ParsedExtension, X509Certificate};
use yaml_rust::YamlLoader;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertificateInfo {
    pub path: PathBuf,
    pub subject: String,
    pub issuer: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub serial: String,
    pub fingerprint: String,
    pub is_ca: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_error: Option<String>,
}
// Add schema-friendly version
#[derive(Debug, Serialize, ToSchema)]
pub struct CertificateInfoSchema {
    pub path: String,
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub serial: String,
    pub fingerprint: String,
    pub is_ca: bool,
    pub last_verified: Option<String>,
    pub verification_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeTrustInfo {
    pub node_ip: String,
    pub certificates: Vec<CertificateInfo>,
    pub trust_chain_valid: bool,
    pub permissions_valid: bool,
    pub expiring_soon: Vec<String>,
    pub last_checked: DateTime<Utc>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct NodeTrustInfoSchema {
    pub node_ip: String,
    pub certificates: Vec<CertificateInfoSchema>,
    pub trust_chain_valid: bool,
    pub permissions_valid: bool,
    pub expiring_soon: Vec<String>,
    pub last_checked: String,
}

pub struct CertificateDiscovery {
    pub trust_store: Arc<RwLock<HashMap<String, NodeTrustInfo>>>,
    verification_interval: Duration,
}

impl CertificateDiscovery {
    pub fn new() -> Self {
        Self {
            trust_store: Arc::new(RwLock::new(HashMap::new())),
            verification_interval: Duration::hours(24),
        }
    }

    pub async fn discover_certificates(
        &self,
        base_path: &Path,
        cert_manager: &mut CertManager,
    ) -> io::Result<Vec<CertificateInfo>> {
        let mut certificates = Vec::new();

        // Ensure the base path exists and is a directory
        if !base_path.exists() {
            cert_manager.log(&format!("Path does not exist: {}", base_path.display()));
            return Ok(certificates);
        }

        if !base_path.is_dir() {
            cert_manager.log(&format!("Path is not a directory: {}", base_path.display()));
            return Ok(certificates);
        }

        // Create multiple explicit glob patterns
        let patterns = vec![
            format!("{}/**/*.crt", base_path.display()),
            format!("{}/**/*.pem", base_path.display()),
            format!("{}/**/*.cert", base_path.display()),
            // Add more explicit patterns if needed
        ];

        let mut processed_paths = 0;
        let mut total_entries = 0;

        for pattern in patterns {
            cert_manager.log(&format!(
                "Searching for certificates with pattern: {}",
                pattern
            ));

            let glob_results = match glob::glob(&pattern) {
                Ok(results) => results,
                Err(e) => {
                    cert_manager.log(&format!("Glob pattern error: {}", e));
                    continue;
                }
            };

            for entry in glob_results {
                total_entries += 1;
                match entry {
                    Ok(path) => {
                        processed_paths += 1;
                        cert_manager.log(&format!(
                            "Processing potential certificate file: {:?}",
                            path
                        ));

                        match self.analyze_certificate(&path).await {
                            Ok(cert_info) => {
                                cert_manager.log(&format!(
                                    "Discovered valid certificate: {} at {:?}",
                                    cert_info.subject, path
                                ));
                                certificates.push(cert_info);
                            }
                            Err(e) => {
                                cert_manager.log(&format!(
                                    "Error analyzing certificate at {:?}: {}",
                                    path, e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        cert_manager.log(&format!("Glob entry error: {}", e));
                    }
                }
            }
        }

        cert_manager.log(&format!(
        "Certificate discovery complete. Total entries: {}, Paths processed: {}, Certificates found: {}",
        total_entries,
        processed_paths,
        certificates.len()
    ));

        Ok(certificates)
    }

    pub async fn analyze_certificate(&self, path: &Path) -> io::Result<CertificateInfo> {
        let cert_pem = fs::read(path)?;

        let cert_der = if cert_pem.starts_with(b"-----BEGIN CERTIFICATE-----") {
            openssl::x509::X509::from_pem(&cert_pem)
                .and_then(|cert| Ok(cert.to_der()?))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            cert_pem
        };

        let (_remainder, cert) = X509Certificate::from_der(&cert_der)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();

        let not_before = chrono::Utc
            .timestamp_opt(cert.validity().not_before.timestamp() as i64, 0)
            .single()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid not_before timestamp")
            })?;
        let not_after = chrono::Utc
            .timestamp_opt(cert.validity().not_after.timestamp() as i64, 0)
            .single()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid not_after timestamp")
            })?;

        let is_ca = cert
            .extensions()
            .iter()
            .find_map(|ext| match ext.parsed_extension() {
                ParsedExtension::BasicConstraints(bc) => Some(bc.ca),
                _ => None,
            })
            .unwrap_or(false);

        Ok(CertificateInfo {
            path: path.to_path_buf(),
            subject,
            issuer,
            not_before,
            not_after,
            serial: hex::encode(cert.raw_serial()),
            fingerprint: hex::encode(openssl::hash::hash(
                openssl::hash::MessageDigest::sha256(),
                &cert_der,
            )?),
            is_ca,
            last_verified: Some(Utc::now()),
            verification_error: None,
        })
    }

    pub async fn start_periodic_verification(&self, nodes: Vec<String>, _ssh_key: String) {
        let trust_store = Arc::clone(&self.trust_store);
        let verification_interval = self.verification_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                verification_interval.num_seconds() as u64,
            ));

            loop {
                interval.tick().await;

                for node in &nodes {
                    let node_info = {
                        if let Some(info) = trust_store.read().await.get(node) {
                            info.clone()
                        } else {
                            continue;
                        }
                    };

                    let mut updated_info = node_info.clone();
                    updated_info.last_checked = Utc::now();

                    // Verify certificates
                    for cert in &mut updated_info.certificates {
                        if let Ok(cert_pem) = fs::read(&cert.path) {
                            let verified = verify_certificate(&cert_pem).is_ok();
                            if !verified {
                                updated_info.trust_chain_valid = false;
                            }
                        }
                    }

                    // Update store - removed the if let Ok pattern
                    let mut store = trust_store.write().await;
                    store.insert(node.clone(), updated_info);
                }
            }
        });
    }

    pub async fn validate_certificate_chain(
        &self,
        cert_path: &Path,
        ca_path: &Path,
    ) -> io::Result<bool> {
        let output = Command::new("openssl")
            .args(&[
                "verify",
                "-CAfile",
                &ca_path.to_string_lossy(),
                &cert_path.to_string_lossy(),
            ])
            .output()?;

        Ok(output.status.success())
    }

    pub async fn check_certificate_expiration(&self, cert_info: &CertificateInfo) -> CertStatus {
        let now = Utc::now();
        let thirty_days = Duration::days(30);

        let (status, last_updated) = if cert_info.not_after < now {
            ("Expired", Some(cert_info.not_after.to_rfc3339()))
        } else if cert_info.not_after - now < thirty_days {
            ("ExpiringSoon", Some(cert_info.not_after.to_rfc3339()))
        } else {
            ("Valid", Some(cert_info.not_after.to_rfc3339()))
        };

        CertStatus {
            cert_type: cert_info.subject.clone(),
            status: status.to_string(),
            last_updated,
        }
    }

    pub async fn validate_node_trust(
        &self,
        node_ip: &str,
        certs: Vec<CertificateInfo>,
    ) -> io::Result<()> {
        let mut node_info = NodeTrustInfo {
            node_ip: node_ip.to_string(),
            certificates: certs.clone(),
            trust_chain_valid: true,
            permissions_valid: true,
            expiring_soon: Vec::new(),
            last_checked: Utc::now(),
        };

        for cert in &certs {
            if cert.is_ca {
                continue;
            }

            if let Some(ca_cert) = self.find_issuing_ca(&cert.issuer).await {
                if !self
                    .validate_certificate_chain(&cert.path, &ca_cert.path)
                    .await?
                {
                    node_info.trust_chain_valid = false;
                }
            }

            let cert_status = self.check_certificate_expiration(cert).await;
            if cert_status.status == "ExpiringSoon" {
                node_info.expiring_soon.push(cert.subject.clone());
            }
        }

        let mut store = self.trust_store.write().await;
        store.insert(node_ip.to_string(), node_info);
        Ok(())
    }
    async fn find_issuing_ca(&self, issuer: &str) -> Option<CertificateInfo> {
        let store = self.trust_store.read().await;
        store
            .values()
            .flat_map(|node| &node.certificates)
            .find(|cert| cert.is_ca && cert.subject == issuer)
            .cloned()
    }

    // Fix for the trust_store write issue
    pub async fn update_trust_store(&self, node_ip: String, info: NodeTrustInfo) -> io::Result<()> {
        let mut store = self.trust_store.write().await;
        store.insert(node_ip, info);
        Ok(())
    }

    // Helper method to get trust store contents
    pub async fn get_trust_store_contents(&self) -> HashMap<String, NodeTrustInfo> {
        self.trust_store.read().await.clone()
    }

    async fn extract_clusters(&self, yaml: &yaml_rust::Yaml) -> Vec<ClusterConfig> {
        yaml["clusters"]
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|cluster| {
                Some(ClusterConfig {
                    name: cluster["name"].as_str()?.to_string(),
                    server: cluster["cluster"]["server"].as_str()?.to_string(),
                    certificate_authority: cluster["cluster"]["certificate-authority"]
                        .as_str()
                        .map(String::from),
                })
            })
            .collect()
    }

    async fn extract_users(&self, yaml: &yaml_rust::Yaml) -> Vec<UserConfig> {
        yaml["users"]
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|user| {
                Some(UserConfig {
                    name: user["name"].as_str()?.to_string(),
                    client_certificate: user["user"]["client-certificate"]
                        .as_str()
                        .map(String::from),
                    client_key: user["user"]["client-key"].as_str().map(String::from),
                })
            })
            .collect()
    }

    async fn extract_contexts(&self, yaml: &yaml_rust::Yaml) -> Vec<ContextConfig> {
        yaml["contexts"]
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|context| {
                Some(ContextConfig {
                    name: context["name"].as_str()?.to_string(),
                    cluster: context["context"]["cluster"].as_str()?.to_string(),
                    user: context["context"]["user"].as_str()?.to_string(),
                })
            })
            .collect()
    }

    pub async fn import_kubeconfig(&self, path: &Path) -> io::Result<KubeConfig> {
        let content = fs::read_to_string(path)?;
        let yaml = YamlLoader::load_from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(KubeConfig {
            clusters: self.extract_clusters(&yaml[0]).await,
            users: self.extract_users(&yaml[0]).await,
            contexts: self.extract_contexts(&yaml[0]).await,
        })
    }
}

fn verify_certificate(cert_pem: &[u8]) -> io::Result<()> {
    // Basic certificate verification logic
    if cert_pem.starts_with(b"-----BEGIN CERTIFICATE-----") {
        openssl::x509::X509::from_pem(cert_pem)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid certificate format",
        ))
    }
}
