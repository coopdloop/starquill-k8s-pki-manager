// types.rs
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // Enable debug mode
    #[arg(short, long)]
    pub debug: bool,

    // Specify custom config path
    #[arg(short, long, default_value = "cluster_config.json")]
    pub config: String,
}

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal,
    EditConfig,
    ViewLogs,
    Confirmation,
}

#[derive(Clone)]
pub struct ConfirmationDialog {
    pub message: String,
    pub callback: ConfirmationCallback,
}

#[derive(Clone)]
pub enum ConfirmationCallback {
    KubernetesCA,
    CAChain,
    DistributePending,
    RootCA,
    AutomateAll, // Add other confirmation types as needed
    VerifyChains,
}

pub enum ScrollDirection {
    Up,
    Down,
    PageUp,
    PageDown,
    Bottom,
    Top,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CertificateStatus {
    pub cert_type: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub generated: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub distributed: Option<DateTime<Utc>>,
    pub path: String,
    pub hosts: Vec<String>,
    pub verified: Option<bool>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_verified: Option<DateTime<Utc>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CertTracker {
    pub certificates: Vec<CertificateStatus>,
}

impl CertTracker {
    pub fn new() -> Self {
        Self {
            certificates: Vec::new(),
        }
    }

    pub fn add_certificate(&mut self, cert_type: &str, path: &str, hosts: Vec<String>) {
        // Check if certificate already exists
        if let Some(existing) = self
            .certificates
            .iter_mut()
            .find(|c| c.cert_type == cert_type)
        {
            existing.generated = Utc::now();
            existing.distributed = None;
            existing.path = path.to_string();
            existing.hosts = hosts;
        } else {
            self.certificates.push(CertificateStatus {
                cert_type: cert_type.to_string(),
                generated: Utc::now(),
                distributed: None,
                path: path.to_string(),
                hosts,
                verified: None,
                last_verified: None,
            });
        }
    }
    pub fn mark_verified(&mut self, cert_type: &str, verified: bool) {
        if let Some(cert) = self
            .certificates
            .iter_mut()
            .find(|c| c.cert_type == cert_type)
        {
            cert.verified = Some(verified);
        }
    }

    pub fn mark_distributed(&mut self, cert_type: &str) {
        if let Some(cert) = self
            .certificates
            .iter_mut()
            .find(|c| c.cert_type == cert_type)
        {
            cert.distributed = Some(Utc::now());
        }
    }

    pub fn get_undistributed(&self) -> Vec<&CertificateStatus> {
        self.certificates
            .iter()
            .filter(|cert| cert.distributed.is_none())
            .filter(|cert|!cert.cert_type.contains("root-ca"))
            .collect()
    }
}
