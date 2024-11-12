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
pub struct CertificateConfig {
    pub cert_type: CertificateType,
    pub common_name: String,
    pub organization: Option<String>,
    pub validity_days: u32,
    pub key_size: u32,
    pub output_dir: PathBuf,
    pub alt_names: Vec<String>,
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClusterEndpoints {
    pub control_plane: String,
    pub worker_nodes: Vec<String>,
}
