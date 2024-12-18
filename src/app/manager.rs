// src/app/manager.rs
use crate::cert::verification::CertificateVerifier;
use crate::cert::{
    CertificateConfig, CertificateOperations, CertificateType, ClusterEndpoints,
    ControllerCertGenerator, ControllerManagerGenerator, NodeCertGenerator,
    ServiceAccountGenerator,
};
use crate::config::{ClusterConfig, ConfigEditor};
use crate::discovery::{CertificateDiscovery, CertificateInfo, NodeTrustInfo};
use crate::kubeconfig::{EncryptionConfigGenerator, KubeConfigGenerator};
use crate::metrics::MetricsCollector;
use crate::types::{
    ActiveSection, AppMode, CertTracker, ConfirmationCallback, ConfirmationDialog, ScrollDirection,
};
use crate::ui;
use crate::utils::logging::Logger;
use crate::web::WebServerState;

use chrono::Local;
use crossterm::event::KeyCode;
use glob::glob;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::{fs, io, path::PathBuf};
use utoipa::ToSchema;

// #[derive(Clone)]
pub struct CertManager {
    pub config: ClusterConfig,
    pub current_operation: String,
    pub logs: Vec<String>,
    pub selected_menu: usize,
    pub menu_items: Vec<String>,
    pub mode: AppMode,
    pub config_editor: ConfigEditor,
    pub debug: bool,
    pub log_scroll: usize,
    pub menu_scroll: usize,
    pub cert_status_scroll: usize,
    pub trust_info_scroll: usize,
    pub active_section: ActiveSection,
    pub confirmation_dialog: Option<ConfirmationDialog>,
    pub cert_tracker: CertTracker,
    pub web_state: Arc<RwLock<WebServerState>>,
    cert_ops: Option<CertificateOperations>,
    pub metrics_collector: Option<MetricsCollector>,
    pub metrics_enabled: bool,
    // log_receiver: Receiver<String>,
    // log_sender: Sender<String>,
    log_receiver: tokio::sync::mpsc::Receiver<String>,
    log_sender: tokio::sync::mpsc::Sender<String>,
    pub kubeconfig_generator: Option<KubeConfigGenerator>,
    pub encryption_generator: Option<EncryptionConfigGenerator>,
    pub trust_store: Option<HashMap<String, NodeTrustInfo>>,
}

#[derive(Clone)]
pub struct OperationsLogger {
    sender: tokio::sync::mpsc::Sender<String>,
    debug: bool,
}

impl OperationsLogger {
    fn new(sender: tokio::sync::mpsc::Sender<String>, debug: bool) -> Self {
        Self { sender, debug }
    }
}

impl Logger for OperationsLogger {
    fn log(&mut self, message: &str) {
        let _ = self.sender.send(message.to_string());
    }

    fn debug_log(&mut self, message: &str) {
        if self.debug {
            let _ = self.sender.send(format!("[DEBUG] {}", message));
        }
    }
}

#[derive(Clone, Serialize, ToSchema)]
pub struct ClusterInfo {
    #[schema(example = "Control Plane Node")]
    pub control_plane: NodeInfo,
    pub workers: Vec<NodeInfo>,
    pub connectivity: ConnectivityStatus,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct ConnectivityStatus {
    pub unreachable_nodes: Vec<String>,
    pub last_checked: String,
    pub total_nodes: usize,
    pub available_nodes: usize,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct NodeInfo {
    #[schema(example = "10.0.0.1")]
    pub ip: String,
    pub certs: Vec<CertStatus>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct CertStatus {
    #[schema(example = "kube-apiserver")]
    pub cert_type: String,
    #[schema(example = "Distributed")]
    pub status: String,
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub last_updated: Option<String>,
}

impl Default for CertManager {
    fn default() -> Self {
        Self::empty()
    }
}

impl CertManager {
    /// Creates a new [`CertManager`].
    fn empty() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        let menu_items = vec![
            "Generate Root CA".to_string(),
            "Generate Kubernetes CA".to_string(),
            "Generate API Server Cert".to_string(),
            "Generate Node Certs".to_string(),
            "Generate Service Account Keys".to_string(),
            "Generate Controller Manager Cert".to_string(),
            "Generate Kubeconfigs".to_string(),       // kubeconifg
            "Generate Encryption Config".to_string(), // kubeconfig
            "Edit Configuration".to_string(),
            "Save Configuration".to_string(),
            "Verify Certificates".to_string(),
            "Exit".to_string(),
            "Distribute Pending Certificates".to_string(),
            "Save Certificate Status".to_string(),
            "Automate all".to_string(),
        ];

        Self {
            config: ClusterConfig::default(),
            current_operation: String::new(),
            logs: Vec::new(),
            selected_menu: 0,
            menu_items,
            mode: AppMode::Normal,
            config_editor: ConfigEditor::new(&ClusterConfig::default()),
            debug: false,
            log_scroll: 0,
            menu_scroll: 0,
            cert_status_scroll: 0,
            trust_info_scroll: 0,
            active_section: ActiveSection::Menu,
            confirmation_dialog: None,
            cert_tracker: CertTracker::new(),
            web_state: Arc::default(),
            cert_ops: None,
            metrics_collector: None,
            metrics_enabled: false,
            log_receiver: receiver,
            log_sender: sender,
            kubeconfig_generator: None,
            encryption_generator: None,
            trust_store: None,
        }
    }

    pub fn new(config: ClusterConfig, debug: bool, web_state: Arc<RwLock<WebServerState>>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        let mut manager = Self {
            config_editor: ConfigEditor::new(&config),
            config,
            current_operation: String::new(),
            logs: Vec::new(),
            selected_menu: 0,
            menu_items: vec![
                "Generate Root CA".to_string(),
                "Generate Kubernetes CA".to_string(),
                "Generate API Server Cert".to_string(),
                "Generate Node Certs".to_string(),
                "Generate Service Account Keys".to_string(),
                "Generate Controller Manager Cert".to_string(),
                "Generate Kubeconfigs".to_string(), // kubeconifg
                "Generate Encryption Config".to_string(), // kubeconfig
                "Edit Configuration".to_string(),
                "Save Configuration".to_string(),
                "Verify Certificates".to_string(),
                "Exit".to_string(),
                "Distribute Pending Certificates".to_string(),
                "Save Certificate Status".to_string(),
                "Import Existing Certificates".to_string(), // Discover
                "Automate all".to_string(),
            ],
            mode: AppMode::Normal,
            debug,
            log_scroll: 0,
            menu_scroll: 0,
            cert_status_scroll: 0,
            trust_info_scroll: 0,
            active_section: ActiveSection::Menu,
            confirmation_dialog: None,
            cert_tracker: CertTracker::new(),
            web_state,
            cert_ops: None,
            metrics_collector: None,
            metrics_enabled: false,
            log_receiver: receiver,
            log_sender: sender,
            kubeconfig_generator: None,
            encryption_generator: None,
            trust_store: None,
        };
        manager.init_cert_ops();
        manager
    }

    // Init and validate automatic
    pub async fn initialize(&mut self) -> io::Result<()> {
        // Auto-discover existing certificates
        match self.import_existing_certificates().await {
            Ok(_) => self.log("Attempted import of existing certificates"),
            Err(e) => self.log(&format!("Certificate import failed: {}", e)),
        }

        let discovery = CertificateDiscovery::new();
        self.trust_store = Some(discovery.get_trust_store_contents().await);

        Ok(())
    }

    // Add initialization method for generators
    pub fn init_generators(&mut self) {
        self.kubeconfig_generator = Some(KubeConfigGenerator::new(
            self.config.control_plane.clone(),
            PathBuf::from("kubeconfig"),
            PathBuf::from("certs/kubernetes-ca/ca-chain.crt"),
        ));

        self.encryption_generator = Some(EncryptionConfigGenerator::new(PathBuf::from(
            "encryption-config.yaml",
        )));
    }

    fn track_kubeconfig(&mut self, config_name: &str, node: &str) {
        self.cert_tracker.add_certificate(
            &format!("kubeconfig-{}", config_name),
            &format!("kubeconfig/{}.conf", config_name),
            vec![node.to_string()],
        );
    }

    pub fn enable_metrics(&mut self, kubeconfig_path: String) {
        self.metrics_enabled = true;
        self.metrics_collector = Some(MetricsCollector::new(true, kubeconfig_path));
    }

    pub fn disable_metrics(&mut self) {
        self.metrics_enabled = false;
        self.metrics_collector = None;
    }

    pub fn generate_all_kubeconfigs(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Kubeconfigs");
        self.log("Starting kubeconfig generation...");

        // Initialize generator if needed
        if self.kubeconfig_generator.is_none() {
            self.init_generators();
        }

        // Clone the values we need upfront
        let control_plane = self.config.control_plane.clone();
        let worker_nodes = self.config.worker_nodes.clone();

        // Generate admin kubeconfig
        {
            let generator = self.kubeconfig_generator.as_ref().unwrap();
            generator.generate_kubeconfig("admin", "default-admin")?;
        }
        self.track_kubeconfig("admin", &control_plane);

        // Generate controller-manager kubeconfig
        {
            let generator = self.kubeconfig_generator.as_ref().unwrap();
            generator
                .generate_kubeconfig("controller-manager", "system:kube-controller-manager")?;
        }
        self.track_kubeconfig("controller-manager", &control_plane);

        // Generate scheduler kubeconfig
        {
            let generator = self.kubeconfig_generator.as_ref().unwrap();
            generator.generate_kubeconfig("scheduler", "system:kube-scheduler")?;
        }
        self.track_kubeconfig("scheduler", &control_plane);

        // Generate node kubeconfigs
        for (i, node) in worker_nodes.iter().enumerate() {
            let node_name = format!("node-{}", i + 1);
            let credential_name = format!("system:node:worker-node-{}", i + 1);
            {
                let generator = self.kubeconfig_generator.as_ref().unwrap();
                generator.generate_kubeconfig(&node_name, &credential_name)?;
            }
            self.track_kubeconfig(&node_name, node);
        }

        self.log("Kubeconfig generation completed successfully");
        Ok(())
    }

    pub fn generate_encryption_config(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Encryption Config");
        self.log("Starting encryption config generation...");

        // Initialize generator if needed
        if self.encryption_generator.is_none() {
            self.init_generators();
        }

        let generator = self.encryption_generator.as_ref().unwrap();
        generator.generate_config()?;

        // Track the encryption config for distribution
        self.cert_tracker.add_certificate(
            "encryption-config",
            "encryption-config.yaml",
            vec![self.config.control_plane.clone()],
        );

        self.log("Encryption config generated successfully");
        Ok(())
    }

    fn create_certificate_operations(&self) -> io::Result<CertificateOperations> {
        Ok(CertificateOperations::new(
            Box::new(OperationsLogger::new(self.log_sender.clone(), self.debug)),
            self.config.remote_dir.clone(),
            self.config.remote_user.clone(),
            self.config.ssh_key_path.clone(),
        ))
    }

    pub fn open_web_ui(&mut self) {
        // Create a smaller scope for the web_state read lock
        let url = {
            let web_state = self.web_state.read().unwrap();
            if !web_state.is_running {
                return;
            }
            format!("http://localhost:{}", web_state.port)
        }; // web_state read guard is dropped here

        // Now we can mutably borrow self for logging
        if let Err(e) = open::that(&url) {
            self.log(&format!("Failed to open browser: {}", e));
        }
    }

    pub fn get_cert_ops(&mut self) -> &mut CertificateOperations {
        self.cert_ops
            .as_mut()
            .expect("CertificateOperations not initialized")
    }

    pub fn init_cert_ops(&mut self) {
        self.cert_ops = Some(CertificateOperations::new(
            Box::new(OperationsLogger::new(self.log_sender.clone(), self.debug)),
            self.config.remote_dir.clone(),
            self.config.remote_user.clone(),
            self.config.ssh_key_path.clone(),
        ));
    }

    // Add method to process logs before rendering
    pub fn process_pending_logs(&mut self) {
        while let Ok(message) = self.log_receiver.try_recv() {
            self.log(&message);
        }
    }

    pub fn automate_all(&mut self) -> io::Result<()> {
        self.log("Starting automated certificate generation and distribution...");

        // 1. Clean up existing certificates
        self.clean_up()?;

        // 2. Generate Root CA
        self.generate_root_ca()?;

        // 3. Generate Kubernetes CA
        self.generate_kubernetes_cert()?;

        // 4. Generate Control Plane certificates
        self.generate_control_plane_certs()?;

        // 5. Generate Worker Node certificates
        self.generate_worker_node_certs()?;

        // 6. Generate Service Account Keys
        self.generate_service_account_keys()?;

        // 7. Generate Kubeconfigs
        self.generate_all_kubeconfigs()?;

        // 8. Generate Encryption Config
        self.generate_encryption_config()?;

        // Distribute everything at once
        self.confirmation_dialog = Some(ConfirmationDialog {
            message: "Do you want to distribute all generated certificates and configs?"
                .to_string(),
            callback: ConfirmationCallback::DistributePending,
        });
        self.mode = AppMode::Confirmation;

        Ok(())
    }

    fn get_cluster_endpoints(&self) -> ClusterEndpoints {
        ClusterEndpoints {
            control_plane: self.config.control_plane.clone(),
            worker_nodes: self.config.worker_nodes.clone(),
        }
    }

    pub fn generate_control_plane_certs(&mut self) -> io::Result<()> {
        self.log("Generating control plane certificates...");

        // Clone all needed values upfront
        let endpoints = self.get_cluster_endpoints();
        let cert_ops = self.get_cert_ops();

        let mut generator = ControllerCertGenerator::new(endpoints, cert_ops);

        // Generate certificates
        generator.generate_api_server_cert()?;
        generator.generate_controller_manager_cert()?;
        generator.generate_scheduler_cert()?;

        self.generate_kubelet_client_cert()?;
        self.generate_service_account_keys()?;

        self.log("Control plane certificates generated successfully");
        Ok(())
    }

    pub fn generate_root_ca(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Root CA");
        let control_plane = self.config.control_plane.clone();
        let hosts = self.get_all_hosts();

        match self.get_cert_ops().setup_ca_certificates(&[&control_plane]) {
            Ok(_) => {
                self.log("Root CA and Kubernetes CA certificates generated successfully");

                // Add certificates to tracker
                self.cert_tracker.add_certificate(
                    "root-ca",
                    "root-ca/ca.crt",
                    // vec![self.config.control_plane.clone()],
                    hosts.clone(),
                );
                self.cert_tracker.add_certificate(
                    "ca.crt",
                    "kubernetes-ca/ca.crt",
                    // vec![self.config.control_plane.clone()],
                    hosts.clone(),
                );
                self.cert_tracker.add_certificate(
                    "ca.key",
                    "kubernetes-ca/ca.key",
                    // vec![self.config.control_plane.clone()],
                    hosts.clone(),
                );
                self.cert_tracker.add_certificate(
                    "ca-chain",
                    "kubernetes-ca/ca-chain.crt",
                    // vec![self.config.control_plane.clone()],
                    hosts.clone(),
                );

                self.cert_tracker.mark_verified("root-ca", true);
                self.cert_tracker.mark_verified("ca.crt", true);
                self.cert_tracker.mark_verified("ca.key", true);
                self.cert_tracker.mark_verified("ca-chain", true);

                // Optional Add confirmation dialog for distributing CA chain
                self.confirmation_dialog = Some(ConfirmationDialog {
                    message: "Do you want to distribute the generated CA certificates?".to_string(),
                    callback: ConfirmationCallback::DistributePending,
                });
                self.mode = AppMode::Confirmation;

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to generate CA certificates: {}", e);
                self.log(&error_msg);
                Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
            }
        }
    }

    pub fn generate_kubernetes_cert(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Kubernetes CA");
        let control_plane = self.config.control_plane.clone();

        let config = CertificateConfig {
            cert_type: CertificateType::KubernetesCA,
            common_name: "kubernetes-ca".to_string(),
            organization: Some("Kubernetes".to_string()),
            validity_days: 3650,
            key_size: 2048,
            output_dir: PathBuf::from("certs/kubernetes-ca"),
            alt_names: vec![],
            key_usage: vec![
                "critical".to_string(),
                "keyCertSign".to_string(),
                "cRLSign".to_string(),
            ],
            extended_key_usage: vec![],
            country: Some("US".to_string()),
            state: Some("Columbia".to_string()),
            locality: Some("Columbia".to_string()),
        };

        self.get_cert_ops().generate_cert(
            "root-ca",
            "certs/root-ca",
            &config,
            &[&control_plane],
        )?;

        // Create CA chain
        self.create_kubernetes_ca_chain()?;

        self.cert_tracker.add_certificate(
            "ca.crt",
            "kubernetes-ca/ca.crt",
            vec![self.config.control_plane.clone()],
        );

        self.cert_tracker.add_certificate(
            "ca.key",
            "kubernetes-ca/ca.key",
            vec![self.config.control_plane.clone()],
        );

        Ok(())
    }

    pub fn generate_kubelet_client_cert(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Kubelet Client Certificate");
        let control_plane = self.config.control_plane.clone();

        let config = CertificateConfig {
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
            country: Some("US".to_string()),
            state: Some("Columbia".to_string()),
            locality: Some("Columbia".to_string()),
        };

        self.get_cert_ops().generate_cert(
            "kube-apiserver-kubelet-client",
            "certs/kubernetes-ca",
            &config,
            &[&control_plane],
        )?;

        self.set_current_operation("Generated Kubelet Client Certificate");

        Ok(())
    }

    pub fn generate_controller_manager_cert(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Controller Manager Certificate");

        let control_plane = self.config.control_plane.clone();
        let cert_ops = self.get_cert_ops();

        let mut generator = ControllerManagerGenerator::new(cert_ops);

        match generator.generate_certificate(&control_plane) {
            Ok(_) => {
                self.cert_tracker.add_certificate(
                    "Controller Manager",
                    "certs/controller-manager/controller-manager.crt",
                    vec![self.config.control_plane.clone()],
                );
                self.cert_tracker.mark_verified("Controller Manager", true);
                Ok(())
            }
            Err(e) => {
                self.log(&format!(
                    "Failed to generate Controller Manager certificate: {}",
                    e
                ));
                Err(e)
            }
        }
    }

    pub fn generate_service_account_keys(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Service Account Keys");
        let cert_ops = self.get_cert_ops();

        let mut sa_generator =
            ServiceAccountGenerator::new(PathBuf::from("certs/service-account"), cert_ops);

        // Generate keys
        sa_generator.generate_service_account_keys()?;

        // Add to certificate tracker
        self.cert_tracker.add_certificate(
            "SA Public Key",
            "certs/service-account/sa.pub",
            vec![self.config.control_plane.clone()],
        );
        self.cert_tracker.add_certificate(
            "SA Private Key",
            "certs/service-account/sa.key",
            vec![self.config.control_plane.clone()],
        );

        // Mark as verified and distributed
        self.cert_tracker.mark_verified("SA Public Key", true);
        self.cert_tracker.mark_verified("SA Private Key", true);
        // self.cert_tracker.mark_distributed("SA Public Key");
        // self.cert_tracker.mark_distributed("SA Private Key");

        Ok(())
    }

    pub fn get_all_hosts(&self) -> Vec<String> {
        let mut hosts = vec![self.config.control_plane.clone()];
        hosts.extend(self.config.worker_nodes.clone());
        hosts
    }

    fn clean_up(&mut self) -> io::Result<()> {
        self.log("Starting cleanup process...");

        // Find all directories containing serial or index.txt files
        let patterns = ["./*/serial", "./*/index.txt"];
        let mut dirs_to_clean = std::collections::HashSet::new();

        for pattern in patterns.iter() {
            for entry in glob(pattern).expect("Failed to read glob pattern") {
                if let Ok(path) = entry {
                    if let Some(dir) = path.parent() {
                        dirs_to_clean.insert(dir.to_path_buf());
                    }
                }
            }
        }

        // Clean each directory
        for dir in dirs_to_clean {
            self.clean_directory(&dir)?;
        }

        self.log("Cleanup completed successfully");
        Ok(())
    }

    fn clean_directory(&mut self, dir: &Path) -> io::Result<()> {
        let dir_str = dir.to_string_lossy();
        self.debug_log(&format!("Cleaning directory: {}", dir_str));

        // Reset serial file
        fs::write(dir.join("serial"), "01")?;
        self.debug_log(&format!(
            "Recreated {}/serial with default value 01",
            dir_str
        ));

        // Reset index.txt
        fs::write(dir.join("index.txt"), "")?;
        self.debug_log(&format!("Recreated {}/index.txt as an empty file", dir_str));

        // Remove old files
        let old_files = [dir.join("index.txt.old"), dir.join("serial.old")];
        for file in old_files.iter() {
            if file.exists() {
                fs::remove_file(file)?;
            }
        }
        self.debug_log(&format!(
            "Removed old serial and index.txt files in {}",
            dir_str
        ));

        // Remove certificate files
        let extensions = [".pem", ".key", ".crt", ".csr"];
        for ext in extensions.iter() {
            let pattern = format!("{}/**/*{}", dir.display(), ext);
            for entry in glob(&pattern).unwrap().filter_map(Result::ok) {
                fs::remove_file(entry)?;
            }
        }
        self.debug_log(&format!("Removed certificate files in {}", dir_str));

        Ok(())
    }

    pub fn verify_certificates(&mut self) -> io::Result<()> {
        self.set_current_operation("Verifying All Certificates");
        self.log("Starting comprehensive certificate verification...");

        let mut verifier = CertificateVerifier::new(
            Box::new(OperationsLogger::new(self.log_sender.clone(), self.debug)),
            self.config.remote_user.clone(),
            self.config.remote_dir.clone(),
            self.config.ssh_key_path.clone(),
        );

        // Clone the certificates to avoid borrowing issues
        let certificates = self.cert_tracker.certificates.clone();

        // Dynamically verify certificates
        for cert in &certificates {
            // Skip if path doesn't exist
            if !Path::new(&cert.path).exists() {
                self.log(&format!("Certificate path not found: {}", cert.path));
                continue;
            }
            self.log(&format!("Certificate path found: {}", cert.path));

            // Skip verification for non-certificate files
            if cert.cert_type.starts_with("kubeconfig-") || cert.cert_type == "encryption-config" {
                self.log(&format!("Skipping verification for {}", cert.cert_type));
                self.cert_tracker.mark_verified(&cert.cert_type, true);
                continue;
            }

            // Determine CA chain based on certificate type
            let ca_chain = match cert.cert_type.as_str() {
                "root-ca" | "kubernetes-ca" => None, // CA certs don't need verification against another CA
                _ => Some("certs/kubernetes-ca/ca-chain.crt"), // Default CA chain
            };

            // Verify the certificate
            match verifier.verify_certificate(&cert.path, ca_chain) {
                Ok(_) => {
                    self.cert_tracker.mark_verified(&cert.cert_type, true);
                    self.log(&format!("{} verified successfully", cert.cert_type));
                }
                Err(e) => {
                    self.cert_tracker.mark_verified(&cert.cert_type, false);
                    self.log(&format!("{} verification failed: {}", cert.cert_type, e));
                }
            }
        }

        // Verify service account keys if they exist
        if Path::new("certs/service-account").exists() {
            match verifier.verify_service_account_keypair(&PathBuf::from("certs/service-account")) {
                Ok(_) => {
                    self.cert_tracker.mark_verified("SA Public Key", true);
                    self.cert_tracker.mark_verified("SA Private Key", true);
                    self.log("Service Account keys verified successfully");
                }
                Err(e) => {
                    self.cert_tracker.mark_verified("SA Public Key", false);
                    self.cert_tracker.mark_verified("SA Private Key", false);
                    self.log(&format!("Service Account keys verification failed: {}", e));
                }
            }
        }

        // Verify remote certificates for all hosts
        let all_hosts = self.get_all_hosts();
        match verifier.verify_remote_certificates(&all_hosts) {
            Ok(_) => self.log("Remote certificate verification completed successfully"),
            Err(e) => self.log(&format!("Remote certificate verification failed: {}", e)),
        }

        self.log("All certificate verifications completed");
        Ok(())
    }

    pub fn load_certificate_status(&mut self) -> io::Result<()> {
        let status_path = PathBuf::from("certificate_status.json");
        if status_path.exists() {
            let status_str = fs::read_to_string(status_path)?;
            self.cert_tracker = serde_json::from_str(&status_str)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.log("Loaded certificate status from file");
        }
        Ok(())
    }

    pub fn save_certificate_status(&self) -> io::Result<()> {
        let status_path = PathBuf::from("certificate_status.json");
        let status_str = serde_json::to_string_pretty(&self.cert_tracker)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(status_path, status_str)?;
        Ok(())
    }

    pub fn get_certificate_status_info(&self) -> Vec<Line> {
        if self.cert_tracker.certificates.is_empty() {
            return vec![Line::from(vec![Span::styled(
                "No certificates generated yet",
                Style::default().fg(Color::DarkGray),
            )])];
        }

        self.cert_tracker
            .certificates
            .iter()
            .map(|cert| {
                let status_color = if cert.distributed.is_some() {
                    Color::Green
                } else {
                    Color::Yellow
                };

                let verify_color = match cert.verified {
                    Some(true) => Color::Green,
                    Some(false) => Color::Red,
                    None => Color::DarkGray,
                };

                let timestamp = cert
                    .generated
                    .with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                Line::from(vec![
                    Span::styled(
                        format!("{:<20}", cert.cert_type),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!(
                            "{:<12}",
                            if cert.distributed.is_some() {
                                "Distributed"
                            } else if cert.cert_type == "root-ca" {
                                "N/A"
                            } else {
                                "Pending"
                            }
                        ),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(
                        format!(
                            "{:<12}",
                            match cert.verified {
                                Some(true) => "Verified",
                                Some(false) => "Failed",
                                None => "Not Verified",
                            }
                        ),
                        Style::default().fg(verify_color),
                    ),
                    Span::styled(
                        format!("Generated: {}", timestamp),
                        Style::default().fg(Color::Gray),
                    ),
                ])
            })
            .collect()
    }

    fn generate_kubeconfig(&mut self, config_name: &str, credential_name: &str) -> io::Result<()> {
        self.log(&format!("Generating kubeconfig for {}", config_name));

        // Create directory if it doesn't exist
        fs::create_dir_all("kubeconfig")?;

        let kubeconfig_path = format!("kubeconfig/{}.conf", config_name);

        // Get API server endpoint
        let api_server = format!("https://{}:6443", self.config.control_plane);

        // Create kubeconfig using kubectl
        let output = Command::new("kubectl")
            .args(&[
                "config",
                "set-cluster",
                "kubernetes",
                "--certificate-authority=kubernetes/ca.crt",
                &format!("--server={}", api_server),
                &format!("--kubeconfig={}", kubeconfig_path),
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ));
        }

        // Set credentials
        let output = Command::new("kubectl")
            .args(&[
                "config",
                "set-credentials",
                credential_name,
                &format!("--client-certificate={}/{}.crt", config_name, config_name),
                &format!("--client-key={}/{}.key", config_name, config_name),
                &format!("--kubeconfig={}", kubeconfig_path),
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ));
        }

        // Set context
        let output = Command::new("kubectl")
            .args(&[
                "config",
                "set-context",
                "default",
                "--cluster=kubernetes",
                &format!("--user={}", credential_name),
                &format!("--kubeconfig={}", kubeconfig_path),
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ));
        }

        // Use context
        let output = Command::new("kubectl")
            .args(&[
                "config",
                "use-context",
                "default",
                &format!("--kubeconfig={}", kubeconfig_path),
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ));
        }

        self.log(&format!("Generated kubeconfig: {}", kubeconfig_path));
        Ok(())
    }

    pub fn save_config(&self) -> io::Result<()> {
        let config_path = PathBuf::from("cluster_config.json");
        self.config.save_to_file(config_path.to_str().unwrap())
    }

    pub fn get_status_info(&self) -> Vec<Line> {
        vec![
            Line::from(vec![
                Span::styled("Current Operation: ", ui::STATUS_LABEL_STYLE),
                Span::styled(
                    &self.current_operation,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Control Plane: ", ui::STATUS_LABEL_STYLE),
                Span::styled(&self.config.control_plane, ui::LOG_SUCCESS_STYLE),
            ]),
            Line::from(vec![
                Span::styled("Worker Nodes: ", ui::STATUS_LABEL_STYLE),
                Span::styled(self.config.worker_nodes.join(", "), ui::LOG_SUCCESS_STYLE),
            ]),
            Line::from(vec![
                Span::styled("SSH Key: ", ui::STATUS_LABEL_STYLE),
                Span::styled(&self.config.ssh_key_path, ui::LOG_DEBUG_STYLE),
            ]),
        ]
    }

    // Update handle_confirmation
    pub fn handle_confirmation(&mut self, confirmed: bool) -> io::Result<()> {
        if let Some(dialog) = self.confirmation_dialog.take() {
            let mut cert_ops = self.create_certificate_operations()?;
            match dialog.callback {
                ConfirmationCallback::RootCA => {
                    if confirmed {
                        let control_plane = self.config.control_plane.clone();

                        if let Err(e) =
                            cert_ops.copy_to_k8s_paths("certs/root-ca/ca.crt", &control_plane)
                        {
                            self.log(&format!("Failed to distribute Root CA certificates: {}", e));
                        } else {
                            self.log("Root CA certificates distributed successfully");
                            // After successful distribution, prompt for chain distribution
                            self.confirmation_dialog = Some(ConfirmationDialog {
                                message:
                                    "Do you want to create and distribute CA chain certificates?"
                                        .to_string(),
                                callback: ConfirmationCallback::CAChain,
                            });
                            self.cert_tracker.mark_distributed("Root CA");
                            self.mode = AppMode::Confirmation;
                        }
                    } else {
                        self.log("Distribution of Root CA certificates was canceled by the user.");
                    }
                }
                ConfirmationCallback::KubernetesCA => {
                    if confirmed {
                        let all_hosts = self.get_all_hosts();
                        let mut success = true;

                        self.cert_tracker.mark_verified("Kubernetes CA", false);
                        for host in &all_hosts {
                            if let Err(e) =
                                cert_ops.copy_to_k8s_paths("certs/kubernetes-ca/ca.crt", host)
                            {
                                self.log(&format!(
                                    "Failed to distribute Kubernetes CA certificates to {}: {}",
                                    host, e
                                ));
                                success = false;
                            }
                        }
                        if success {
                            self.cert_tracker.mark_distributed("Kubernetes CA");
                            self.cert_tracker.mark_verified("Kubernetes CA", true);
                            self.log("Kubernetes CA certificates distributed successfully");
                        }
                    } else {
                        self.log(
                            "Distribution of Kubernetes CA certificates was canceled by the user.",
                        );
                    }
                }
                ConfirmationCallback::CAChain => {
                    if confirmed {
                        let hosts = self.get_all_hosts();
                        let mut success = true;

                        for host in &hosts {
                            if let Err(e) =
                                cert_ops.copy_to_k8s_paths("certs/kubernetes-ca/ca-chain.crt", host)
                            {
                                self.log(&format!(
                                    "Failed to distribute CA chain certificates to {}: {}",
                                    host, e
                                ));
                                success = false;
                            }
                        }

                        if success {
                            self.log("CA chain certificates created and distributed successfully");
                            self.cert_tracker.mark_distributed("ca-chain");
                        } else {
                            self.log("Failed to distribute some CA chain certificates");
                        }
                    } else {
                        self.log("CA chain certificate distribution was canceled by the user.");
                    }
                }
                ConfirmationCallback::AutomateAll => {
                    if confirmed {
                        match self.automate_all() {
                            Ok(_) => {
                                self.log("Automated certificate generation completed successfully")
                            }
                            Err(e) => self.log(&format!("Automation failed: {}", e)),
                        }
                    } else {
                        self.log("Automation cancelled by user");
                    }
                }
                ConfirmationCallback::VerifyChains => {
                    if confirmed {
                        self.log("Starting verification of distributed certificates...");
                        match self.verify_certificates() {
                            Ok(_) => self.log("Remote verification successful, trust is complete."),
                            Err(e) => self.log(&format!("Verification failed: {}", e)),
                        }
                    }
                }
                ConfirmationCallback::DistributePending => {
                    if confirmed {
                        self.mode = AppMode::Normal;
                        let pending_certs: Vec<(String, String, Vec<String>)> = self
                            .cert_tracker
                            .get_undistributed()
                            .iter()
                            .map(|cert| {
                                (
                                    cert.cert_type.clone(),
                                    cert.path.clone(),
                                    cert.hosts.clone(),
                                )
                            })
                            .collect();

                        self.log(&format!("Distributing {:?} ", pending_certs));
                        if pending_certs.is_empty() {
                            self.log("No certificates pending distribution");
                        } else {
                            for (cert_type, path, hosts) in pending_certs {
                                self.log(&format!("Distributing {} certificate...", cert_type));

                                let mut cert_success = true;

                                if cert_type == "root-ca" {
                                    continue;
                                }

                                for host in hosts {
                                    self.log(&format!("Distributing to host: {}", host));
                                    let source_path = if cert_type.starts_with("kubeconfig-") {
                                        format!("{}", path)
                                    } else if cert_type == "encryption-config" {
                                        "encryption-config.yaml".to_string()
                                    } else {
                                        format!("{}", path)
                                    };

                                    match cert_ops.copy_to_k8s_paths(&source_path, &host) {
                                        Ok(_) => {
                                            self.log(&format!(
                                                "Successfully distributed {} to {}",
                                                cert_type, host
                                            ));
                                        }
                                        Err(e) => {
                                            self.log(&format!(
                                                "Failed to distribute {} to {}: {}",
                                                cert_type, host, e
                                            ));
                                            cert_success = false;
                                        }
                                    }
                                }

                                if cert_success {
                                    self.cert_tracker.mark_distributed(&cert_type);
                                }
                            }
                        }
                    } else {
                        self.log("Distribution of pending certificates cancelled by user");
                    }
                }
            }
        }
        self.mode = AppMode::Normal;
        Ok(())
    }

    fn create_kubernetes_ca_chain(&mut self) -> io::Result<()> {
        self.debug_log("Creating Kubernetes CA chain");

        // Check if root CA exists
        let root_ca_path = "certs/root-ca/ca.crt";
        let kubernetes_ca_path = "certs/kubernetes-ca/ca.crt";
        let chain_path = "certs/kubernetes-ca/ca-chain.crt";

        if !Path::new(root_ca_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Root CA certificate not found. Please generate root CA first.",
            ));
        }

        if !Path::new(kubernetes_ca_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Kubernetes CA certificate not found",
            ));
        }

        // Read both certificates
        let root_ca_content = fs::read_to_string(root_ca_path)?;
        let kubernetes_ca_content = fs::read_to_string(kubernetes_ca_path)?;

        // Create chain file by concatenating both CAs
        let chain_content = format!("{}\n{}", root_ca_content, kubernetes_ca_content);
        fs::write(chain_path, chain_content)?;

        // Verify the chain
        let output = Command::new("openssl")
            .args(&["verify", "-CAfile", root_ca_path, kubernetes_ca_path])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            self.log(&format!("CA chain verification failed: {}", error_msg));
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("CA chain verification failed: {}", error_msg),
            ));
        }

        self.log("CA chain created and verified successfully");
        Ok(())
    }

    pub fn generate_worker_node_certs(&mut self) -> io::Result<()> {
        self.set_current_operation("Generating Worker node certificates.");
        let worker_nodes = self.config.worker_nodes.clone();

        let nodes: Vec<(usize, String)> = worker_nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (i, node.clone()))
            .collect();

        let cert_ops = self.get_cert_ops();
        let mut generator = NodeCertGenerator::new(cert_ops);

        generator.generate_node_certificates(&nodes)?;

        // Generate and distribute kubeconfigs after certificates
        self.generate_worker_kubeconfigs()?;

        Ok(())
    }

    fn generate_worker_kubeconfigs(&mut self) -> io::Result<()> {
        let mut cert_ops = self.create_certificate_operations()?;

        for (i, node) in self.config.worker_nodes.clone().iter().enumerate() {
            let node_name = format!("node-{}", i + 1);
            let credential_name = format!("system:node:{}", node_name);

            // Generate kubeconfig
            self.generate_kubeconfig(&node_name, &credential_name)?;

            // Use cert_ops for distribution
            cert_ops.copy_to_k8s_paths(&format!("kubeconfig/{}.conf", node_name), node)?;
        }
        Ok(())
    }

    pub fn set_current_operation(&mut self, operation: &str) {
        self.current_operation = operation.to_string();
        self.log(&format!("Starting operation: {}", operation));
    }

    pub fn handle_config_edit(&mut self, key: KeyCode) {
        match key {
            KeyCode::Tab => {
                if self.config_editor.is_editing {
                    self.config_editor.handle_tab();
                }
            }
            KeyCode::Enter => {
                if self.config_editor.is_editing {
                    self.config_editor.fields[self.config_editor.current_field] =
                        self.config_editor.editing_value.clone();
                    self.config_editor.is_editing = false;
                    self.config_editor.reset_completions(); // Reset when confirming value
                    self.config_editor.editing_value.clear();
                    self.config_editor.apply_to_config(&mut self.config);
                    self.log("Configuration field updated");
                } else {
                    self.config_editor.is_editing = true;
                    self.config_editor.editing_value =
                        self.config_editor.fields[self.config_editor.current_field].clone();
                }
            }
            KeyCode::Esc => {
                if self.config_editor.is_editing {
                    self.config_editor.is_editing = false;
                    self.config_editor.reset_completions(); // Reset when canceling edit
                    self.config_editor.editing_value.clear();
                    self.log("Edit cancelled");
                } else {
                    self.mode = AppMode::Normal;
                    self.log("Exited configuration mode");
                }
            }
            KeyCode::Up if !self.config_editor.is_editing => {
                self.config_editor.current_field = self
                    .config_editor
                    .current_field
                    .checked_sub(1)
                    .unwrap_or(self.config_editor.fields.len() - 1);
                self.config_editor.reset_completions(); // Reset when changing fields
            }
            KeyCode::Down if !self.config_editor.is_editing => {
                self.config_editor.current_field =
                    (self.config_editor.current_field + 1) % self.config_editor.fields.len();
                self.config_editor.reset_completions(); // Reset when changing fields
            }
            KeyCode::Char(c) if self.config_editor.is_editing => {
                self.config_editor.editing_value.push(c);
                self.config_editor.reset_completions(); // Reset when typing new characters
                self.debug_log(&format!(
                    "Current value: {}",
                    self.config_editor.editing_value
                ));
            }
            KeyCode::Backspace if self.config_editor.is_editing => {
                self.config_editor.editing_value.pop();
                self.config_editor.reset_completions(); // Reset when deleting characters
                self.debug_log(&format!(
                    "Current value: {}",
                    self.config_editor.editing_value
                ));
            }
            _ => {}
        }
    }

    pub async fn import_existing_certificates(&mut self) -> io::Result<()> {
        let discovery = CertificateDiscovery::new();

        // Use full paths to the directories containing certificates
        let paths = [
            Path::new("certs/kubernetes-ca"),
            Path::new("certs/root-ca"),
            Path::new("certs"), // Fallback to full certs directory
            Path::new("kubeconfig"),
            Path::new("pki"),
        ];

        for path in &paths {
            match discovery.discover_certificates(path, self).await {
                Ok(certs) => {
                    for cert in certs {
                        // Standardize certificate names
                        let cert_type = self.determine_cert_type(&cert);

                        self.log(&format!(
                            "Importing certificate: {} (Original subject: {})",
                            cert_type, cert.subject
                        ));

                        self.cert_tracker.add_certificate(
                            &cert_type, // Use standardized name
                            cert.path.to_str().unwrap(),
                            vec![self.config.control_plane.clone()],
                        );
                    }
                }
                Err(e) => self.log(&format!(
                    "Error discovering certificates in {}: {}",
                    path.display(),
                    e
                )),
            }
        }

        self.validate_cluster_trust(&discovery).await?;
        Ok(())
    }

    // Helper function to determine standard certificate type
    pub fn determine_cert_type(&mut self, cert_info: &CertificateInfo) -> String {
        // Check filename first
        let filename = cert_info
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        // Filename-based rules
        if filename.contains("ca-chain") {
            return "ca-chain".to_string();
        }

        if filename.contains("sa.pub") {
            return "sa-public-key".to_string();
        }

        if filename.contains("sa.key") {
            return "sa-private-key".to_string();
        }

        if filename.contains("encryption-config") {
            return "encryption-config".to_string();
        }

        if filename.contains("ca.crt") {
            // Check subject for more precise identification
            let subject_lower = cert_info.subject.to_lowercase();

            if subject_lower.contains("root") {
                return "root-ca".to_string();
            }

            return "ca.crt".to_string();
        }

        if filename.contains("ca.key")  {
            // Check subject for more precise identification
            let subject_lower = cert_info.subject.to_lowercase();
            self.log(&subject_lower);

            return "ca.key".to_string();
        }

        // Subject-based detection for special keys
        let subject_lower = cert_info.subject.to_lowercase();

        if subject_lower.contains("service account") {
            if filename.contains(".key") {
                return "sa-private-key".to_string();
            }
            if filename.contains(".pub") {
                return "sa-public-key".to_string();
            }
        }

        // Controller Manager specific detection
        if subject_lower.contains("system:kube-controller-manager") {
            return "controller-manager".to_string();
        }

        // Fallback to subject-based detection
        if cert_info.subject.contains("Root CA") {
            return "root-ca".to_string();
        }

        if cert_info.subject.contains("Kubernetes Root CA") {
            return "ca".to_string();
        }

        // Default fallback
        if cert_info.is_ca {
            return "ca".to_string();
        }

        // If all else fails, use a generic name with a hash to ensure uniqueness
        format!("cert-{}", &cert_info.fingerprint[..8])
    }

    async fn validate_cluster_trust(&mut self, discovery: &CertificateDiscovery) -> io::Result<()> {
        // Clear existing trust store to start fresh
        let mut trust_store = discovery.get_trust_store_contents().await;
        trust_store.clear();

        // Validate control plane certificates
        let control_plane_certs = discovery
            .discover_certificates(Path::new("certs"), self)
            .await?;
        // Validate control plane trust
        discovery
            .validate_node_trust(&self.config.control_plane, control_plane_certs.clone())
            .await?;

        // Validate worker nodes
        for worker in self.config.worker_nodes.clone() {
            // Try to find worker-specific certificates
            let worker_certs = discovery
                .discover_certificates(Path::new(&format!("certs/node-{}", worker)), self)
                .await
                .unwrap_or(control_plane_certs.clone());

            discovery.validate_node_trust(&worker, worker_certs).await?;
        }

        // Retrieve the updated trust store contents
        let updated_trust_store = discovery.get_trust_store_contents().await;

        for (node, trust_info) in &updated_trust_store {
            self.log(&format!(
                "Node {} trust validation - Trust chain valid: {} && Permissions valid: {}",
                node, trust_info.trust_chain_valid, trust_info.permissions_valid
            ));
            self.log(&format!(
                "  Trust chain valid: {}",
                trust_info.trust_chain_valid
            ));
            self.log(&format!(
                "  Permissions valid: {}",
                trust_info.permissions_valid
            ));

            if !trust_info.expiring_soon.is_empty() {
                self.log("  Certificates expiring soon:");
                for cert in &trust_info.expiring_soon {
                    self.log(&format!("    - {}", cert));
                }
            }
        }

        // Update the local trust store
        self.trust_store = Some(updated_trust_store);

        Ok(())
    }

    pub fn log(&mut self, message: &str) {
        self.logs.push(format!(
            "{}: {}",
            chrono::Local::now().format("%H:%M:%S"),
            message
        ));

        self.scroll_to_bottom()
    }

    fn debug_log(&mut self, message: &str) {
        if self.debug {
            self.log(&format!("[DEBUG] {}", message));
        }
    }

    // Helper method to scroll to bottom explicitly
    pub fn scroll_to_bottom(&mut self) {
        let visible_height = 18;
        if self.logs.len() > visible_height {
            // Set scroll position to show the last 'visible_height' logs
            self.log_scroll = self.logs.len() - visible_height;
        } else {
            // If we have fewer logs than the window height, start from the beginning
            self.log_scroll = 0;
        }
    }

    // Modified scroll_logs to handle bounds better
    pub fn scroll_logs(&mut self, direction: ScrollDirection) {
        let max_scroll = self.logs.len().saturating_sub(1);

        match direction {
            ScrollDirection::Up => {
                if self.log_scroll > 0 {
                    self.log_scroll -= 1;
                }
            }
            ScrollDirection::Down => {
                if self.log_scroll < max_scroll {
                    self.log_scroll += 1;
                }
            }
            ScrollDirection::PageUp => {
                if self.log_scroll > 10 {
                    self.log_scroll -= 10;
                } else {
                    self.log_scroll = 0;
                }
            }
            ScrollDirection::PageDown => {
                if self.log_scroll + 10 < max_scroll {
                    self.log_scroll += 10;
                } else {
                    self.log_scroll = max_scroll;
                }
            }
            ScrollDirection::Bottom => {
                self.log_scroll = max_scroll;
            }
            ScrollDirection::Top => {
                self.log_scroll = 0;
            }
        }
    }
}
