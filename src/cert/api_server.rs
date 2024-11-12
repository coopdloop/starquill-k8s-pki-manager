// src/cert/api_server.rs
use super::types::CertificateConfig;
use super::CertificateType;
use std::path::PathBuf;

pub struct ApiServerCertGenerator {
    control_plane: String,
}

impl ApiServerCertGenerator {
    pub fn new(control_plane: String) -> Self {
        Self { control_plane }
    }

    pub fn get_config(&self) -> CertificateConfig {
        CertificateConfig {
            cert_type: CertificateType::APIServer,
            common_name: "kube-apiserver".to_string(),
            organization: Some("Kubernetes".to_string()),
            validity_days: 375,
            key_size: 2048,
            output_dir: PathBuf::from("certs/kube-apiserver"),
            alt_names: vec![
                "kubernetes".to_string(),
                "kubernetes.default".to_string(),
                "kubernetes.default.svc".to_string(),
                "kubernetes.default.svc.cluster.local".to_string(),
                format!("IP:{}", self.control_plane),
            ],
            key_usage: vec![
                "critical".to_string(),
                "digitalSignature".to_string(),
                "keyEncipherment".to_string(),
            ],
            extended_key_usage: vec!["serverAuth".to_string()],
        }
    }
}
