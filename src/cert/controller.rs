// src/cert/controller.rs

use super::{
    operations::CertificateOperations,
    types::{CertificateConfig, CertificateType, ClusterEndpoints},
};
use std::{io, path::PathBuf};

pub struct ControllerCertGenerator<'a> {
    endpoints: ClusterEndpoints,
    cert_ops: &'a mut CertificateOperations,
}

impl<'a> ControllerCertGenerator<'a> {
    pub fn new(endpoints: ClusterEndpoints, cert_ops: &'a mut CertificateOperations) -> Self {
        Self {
            endpoints,
            cert_ops,
        }
    }

    pub fn generate_api_server_cert(&mut self) -> io::Result<()> {
        self.cert_ops.log("Generating API Server Certificate");

        let config = self.get_apiserver_config();
        self.cert_ops.generate_cert(
            "kube-apiserver",
            "certs/kubernetes-ca",
            &config,
            &[&self.endpoints.control_plane],
        )?;

        // self.cert_ops.logger
        //     .log("API Server certificate generated successfully");
        Ok(())
    }

    pub fn generate_controller_manager_cert(&mut self) -> io::Result<()> {
        // self.cert_ops.logger.log("Generating Controller Manager Certificate");

        let config = self.get_controller_config();
        self.cert_ops.generate_cert(
            "controller-manager",
            "certs/kubernetes-ca",
            &config,
            &[&self.endpoints.control_plane],
        )?;

        // self.cert_ops
        //     .logger
        //     .log("Controller Manager certificate generated successfully");
        Ok(())
    }

    pub fn generate_scheduler_cert(&mut self) -> io::Result<()> {
        // self.cert_ops.logger.log("Generating Scheduler Certificate");

        let config = self.get_scheduler_config();
        self.cert_ops.generate_cert(
            "scheduler",
            "certs/kubernetes-ca",
            &config,
            &[&self.endpoints.control_plane],
        )?;

        // self.cert_ops
        //     .logger
        //     .log("Scheduler certificate generated successfully");
        Ok(())
    }

    fn get_controller_config(&self) -> CertificateConfig {
        CertificateConfig {
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
        }
    }

    fn get_scheduler_config(&self) -> CertificateConfig {
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

    fn get_apiserver_config(&self) -> CertificateConfig {
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
                format!("IP:{}", self.endpoints.control_plane),
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
