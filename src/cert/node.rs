use super::operations::CertificateOperations;
use super::types::{CertificateConfig, CertificateType};
use super::CertOperationError;
use std::{io, path::PathBuf};

#[derive(Debug)]
pub enum NodeCertError {
    IoError(io::Error),
    CertOperation(CertOperationError),
}

impl From<io::Error> for NodeCertError {
    fn from(error: io::Error) -> Self {
        NodeCertError::IoError(error)
    }
}

impl From<CertOperationError> for NodeCertError {
    fn from(error: CertOperationError) -> Self {
        NodeCertError::CertOperation(error)
    }
}

pub struct NodeCertGenerator<'a> {
    cert_ops: &'a mut CertificateOperations,
}

impl<'a> NodeCertGenerator<'a> {
    pub fn new(cert_ops: &'a mut CertificateOperations) -> Self {
        Self { cert_ops }
    }

    pub fn generate_node_certificates(
        &mut self,
        nodes: &[(usize, String)], // (index, node_address)
    ) -> io::Result<()> {
        self.cert_ops
            .log("Starting worker node certificate generation...");

        for (index, node) in nodes {
            let node_name = format!("node-{}", index + 1);
            self.generate_node_certificate(&node_name, node, *index);
        }

        self.cert_ops
            .log("Worker node certificates generated successfully");
        Ok(())
    }

    fn generate_node_certificate(
        &mut self,
        node_name: &str,
        node: &str,
        index: usize,
    ) -> Result<(), CertOperationError> {
        self.cert_ops
            .log(&format!("Generating certificate for {}", node_name));

        let config = CertificateConfig {
            cert_type: CertificateType::Node(node_name.to_string()),
            common_name: format!("system:node:{}", node_name),
            organization: Some("system:nodes".to_string()),
            validity_days: 375,
            key_size: 2048,
            output_dir: PathBuf::from(format!("certs/{}", node_name)),
            alt_names: vec![
                format!("DNS:{}", node),
                format!("IP:{}", node),
                format!("DNS:node-{}", index + 1),
                format!("DNS:node-{}.cluster.local", index + 1),
            ],
            key_usage: vec![
                "critical".to_string(),
                "digitalSignature".to_string(),
                "keyEncipherment".to_string(),
            ],
            extended_key_usage: vec!["serverAuth".to_string(), "clientAuth".to_string()],
        };

        self.cert_ops
            .generate_cert(node_name, "certs/kubernetes-ca", &config, &[node])
    }
}
