use std::path::PathBuf;

use super::{CertificateConfig, CertificateType};

// src/cert/scheduler.rs
pub struct SchedulerCertGenerator;

impl SchedulerCertGenerator {
    pub fn get_config() -> CertificateConfig {
        CertificateConfig {
            cert_type: CertificateType::Scheduler,
            common_name: "system:kube-scheduler".to_string(),
            organization: Some("system:kube-scheduler".to_string()),
            validity_days: 375,
            key_size: 2048,
            output_dir: PathBuf::from("certs/scheduler"),
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
