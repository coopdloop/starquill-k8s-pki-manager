// src/cert/controller_manager.rs

use super::operations::CertificateOperations;
use super::types::{CertificateConfig, CertificateType};
use std::io;
use std::path::PathBuf;

pub struct ControllerManagerGenerator<'a> {
    cert_ops: &'a mut CertificateOperations,
}

impl<'a> ControllerManagerGenerator<'a> {
    pub fn new(cert_ops: &'a mut CertificateOperations) -> Self {
        Self { cert_ops }
    }

    pub fn generate_certificate(&mut self, control_plane: &str) -> io::Result<()> {
        self.cert_ops
            .log("Generating Controller Manager certificate");

        let config = CertificateConfig {
            cert_type: CertificateType::ControllerManager,
            common_name: "system:kube-controller-manager".to_string(),
            organization: Some("system:kube-controller-manager".to_string()),
            validity_days: 375,
            key_size: 2048,
            output_dir: PathBuf::from("certs/controller-manager"),
            alt_names: vec![],
            key_usage: vec![
                "critical".to_string(),
                "digitalSignature".to_string(),
                "keyEncipherment".to_string(),
            ],
            extended_key_usage: vec!["clientAuth".to_string()],
            country: Some("US".to_string()),
            state: Some("Columbia".to_string()),
            locality: Some("Columbia".to_string()),
        };

        self.cert_ops.generate_cert(
            "controller-manager",
            "certs/kubernetes-ca",
            &config,
            &[control_plane],
        );

        self.cert_ops
            .log("Controller Manager certificate generated successfully");
        Ok(())
    }
}
