use std::path::PathBuf;

use super::{CertificateConfig, CertificateType};

// src/cert/kubelet.rs
pub struct KubeletClientCertGenerator;

impl KubeletClientCertGenerator {
    pub fn get_config() -> CertificateConfig {
        CertificateConfig {
            cert_type: CertificateType::KubeletClient,
            common_name: "kube-apiserver-kubelet-client".to_string(),
            organization: Some("system:masters".to_string()),
            validity_days: 375,
            key_size: 2048,
            output_dir: PathBuf::from("certs/kube-apiserver-kubelet-client"),
            alt_names: vec![],
            key_usage: vec![
                "critical".to_string(),
                "digitalSignature".to_string(),
                "keyEncipherment".to_string(),
            ],
            extended_key_usage: vec!["clientAuth".to_string()],
        }
    }
}
