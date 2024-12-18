// cert/types.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateType {
    RootCA,
    KubernetesCA,
    APIServer,
    KubeletClient,
    ServiceAccount,
    ControllerManager,
    Scheduler,
    Node(String),
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltName {
    pub alt_type: AltNameType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AltNameType {
    DNS,
    IP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateConfig {
    pub cert_type: CertificateType,
    pub common_name: String,
    pub organization: Option<String>,
    pub validity_days: u32,
    pub key_size: u32,
    pub output_dir: PathBuf,
    pub alt_names: Vec<AltName>,  // Changed from Vec<String>
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
    // Optional additional fields to match your OpenSSL config
    pub country: Option<String>,
    pub state: Option<String>,
    pub locality: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClusterEndpoints {
    pub control_plane: String,
    pub worker_nodes: Vec<String>,
}

// Implementation for AltName for easier creation
impl AltName {
    pub fn dns(value: String) -> Self {
        Self {
            alt_type: AltNameType::DNS,
            value,
        }
    }

    pub fn ip(value: String) -> Self {
        Self {
            alt_type: AltNameType::IP,
            value,
        }
    }

    // Helper to format for OpenSSL config
    pub fn to_openssl_format(&self) -> String {
        match self.alt_type {
            AltNameType::DNS => format!("DNS:{}", self.value),
            AltNameType::IP => format!("IP:{}", self.value),
        }
    }
}
