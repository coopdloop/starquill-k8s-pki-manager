// src/app/manager.rs
use crate::cert::verification::CertificateVerifier;
use crate::cert::ControllerManagerGenerator;
use crate::cert::{
    CertificateConfig, CertificateOperations, CertificateType, ClusterEndpoints,
    ControllerCertGenerator, NodeCertGenerator, ServiceAccountGenerator,
};
use crate::config::{ClusterConfig, ConfigEditor};
use crate::types::{
    AppMode, CertTracker, ConfirmationCallback, ConfirmationDialog, ScrollDirection,
};
use crate::utils::logging::Logger;
use base64::{engine::general_purpose, Engine as _};
use chrono::Local;
use crossterm::event::KeyCode;
use glob::glob;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{fs, io, path::PathBuf};

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
    pub confirmation_dialog: Option<ConfirmationDialog>,
    pub cert_tracker: CertTracker,
    cert_ops: Option<CertificateOperations>,
    log_receiver: Receiver<String>,
    log_sender: Sender<String>,
}

#[derive(Clone)]
pub struct OperationsLogger {
    sender: Sender<String>,
    debug: bool,
}

impl OperationsLogger {
    fn new(sender: Sender<String>, debug: bool) -> Self {
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
impl CertManager {
    pub fn new(config: ClusterConfig, debug: bool) -> Self {
        let (sender, receiver) = channel();

        let menu_items = vec![
            "Generate Root CA".to_string(),
            "Generate Kubernetes CA".to_string(),
            "Generate API Server Cert".to_string(),
            "Generate Node Certs".to_string(),
            "Generate Service Account Keys".to_string(),
            "Generate Controller Manager Cert".to_string(),
            "Edit Configuration".to_string(),
            "Save Configuration".to_string(),
            "Verify Certificates".to_string(),
            "Exit".to_string(),
            "Distribute Pending Certificates".to_string(), // New item
            "Save Certificate Status".to_string(),         // New menu item
            "Automate all".to_string(),
        ];

        let mut manager = Self {
            config_editor: ConfigEditor::new(&config),
            config,
            current_operation: String::new(),
            logs: Vec::new(),
            selected_menu: 0,
            menu_items,
            mode: AppMode::Normal,
            debug,
            log_scroll: 0,
            confirmation_dialog: None,
            cert_tracker: CertTracker::new(),
            cert_ops: None,
            log_receiver: receiver,
            log_sender: sender,
        };

        manager.init_cert_ops();
        manager
    }

    fn create_certificate_operations(&self) -> io::Result<CertificateOperations> {
        Ok(CertificateOperations::new(
            Box::new(OperationsLogger::new(self.log_sender.clone(), self.debug)),
            self.config.remote_dir.clone(),
            self.config.remote_user.clone(),
            self.config.ssh_key_path.clone(),
        ))
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
        self.generate_kubeconfigs()?;

        // 8. Generate Encryption Config
        self.generate_encryption_config()?;

        self.log("Certificate generation and distribution completed successfully!");
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
                    "certs/root-ca/ca.crt",
                    // vec![self.config.control_plane.clone()],
                    hosts.clone()
                );
                self.cert_tracker.add_certificate(
                    "kubernetes-ca",
                    "certs/kubernetes-ca/ca.crt",
                    // vec![self.config.control_plane.clone()],
                    hosts.clone()
                );
                self.cert_tracker.add_certificate(
                    "ca-chain",
                    "certs/kubernetes-ca/ca-chain.crt",
                    // vec![self.config.control_plane.clone()],
                    hosts.clone()
                );

                self.cert_tracker.mark_verified("root-ca", true);
                self.cert_tracker.mark_verified("kubernetes-ca", true);
                self.cert_tracker.mark_verified("ca-chain", true);

                // Add confirmation dialog for distributing CA chain
                // self.confirmation_dialog = Some(ConfirmationDialog {
                //     message: "Do you want to distribute the CA chain to all hosts?".to_string(),
                //     callback: ConfirmationCallback::CAChain,
                // });
                // self.mode = AppMode::Confirmation;

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
        };

        self.get_cert_ops().generate_cert(
            "kubernetes-ca",
            "certs/root-ca",
            &config,
            &[&control_plane],
        )?;

        // Create CA chain
        self.create_kubernetes_ca_chain()?;

        self.cert_tracker.add_certificate(
            "Kubernetes CA",
            "certs/kubernetes-ca/ca.crt",
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
        };

        self.get_cert_ops().generate_cert(
            "kube-apiserver-kubelet-client",
            "certs/kubernetes-ca",
            &config,
            &[&control_plane],
        )?;

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
        self.cert_tracker.mark_distributed("SA Public Key");
        self.cert_tracker.mark_distributed("SA Private Key");

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
            // Box::new(self.clone()),
            self.config.remote_user.clone(),
            self.config.remote_dir.clone(),
            self.config.ssh_key_path.clone(),
        );

        // Verify local certificates
        let ca_chain_path = "certs/kubernetes-ca/ca-chain.crt";
        let certificates = [
            ("Root CA", "certs/root-ca/ca.crt", None),
            (
                "Kubernetes CA",
                "certs/kubernetes-ca/ca.crt",
                Some("certs/root-ca/ca.crt"),
            ),
            (
                "API Server",
                "certs/kube-apiserver/kube-apiserver.crt",
                Some(ca_chain_path),
            ),
            (
                "Controller Manager",
                "certs/controller-manager/controller-manager.crt",
                Some(ca_chain_path),
            ),
            (
                "Scheduler",
                "certs/scheduler/scheduler.crt",
                Some(ca_chain_path),
            ),
        ];

        for (name, path, ca_cert) in certificates {
            if Path::new(path).exists() {
                match verifier.verify_certificate(path, ca_cert) {
                    Ok(_) => {
                        self.cert_tracker.mark_verified(name, true);
                        self.log(&format!("{} verified successfully", name));
                    }
                    Err(e) => {
                        self.cert_tracker.mark_verified(name, false);
                        self.log(&format!("{} verification failed: {}", name, e));
                    }
                }
            }
        }

        // Verify remote certificates
        let all_hosts = self.get_all_hosts();
        verifier.verify_remote_certificates(&all_hosts)?;

        // Verify service account keys
        verifier.verify_service_account_keypair(&PathBuf::from("certs/service-account"))?;

        self.log("All certificate verifications completed successfully");
        Ok(())
    }

    fn verify_service_account_keypair(&mut self) -> io::Result<()> {
        self.log("Verifying service account key pair...");

        let output = Command::new("openssl")
            .args(&[
                "rsa",
                "-in",
                "certs/service-account/sa.key",
                "-pubout",
                "-outform",
                "PEM",
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to verify service account key pair",
            ));
        }

        self.log("Service account key pair verified successfully");
        Ok(())
    }

    fn copy_from_remote(&self, host: &str, remote_path: &str, local_path: &str) -> io::Result<()> {
        let ssh_key_path = shellexpand::tilde(&self.config.ssh_key_path).to_string();

        let output = Command::new("scp")
            .args(&[
                "-i",
                &ssh_key_path,
                &format!("{}@{}:{}", self.config.remote_user, host, remote_path),
                local_path,
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to copy {} from {}", remote_path, host),
            ));
        }

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

    pub fn generate_kubeconfigs(&mut self) -> io::Result<()> {
        self.log("Generating kubeconfigs...");

        // Define all the kubeconfig configurations
        let configs = [
            ("admin", "default-admin"),
            ("controller-manager", "system:kube-controller-manager"),
            ("scheduler", "system:kube-scheduler"),
        ];

        // Generate kubeconfigs for control plane components
        for (config_name, credential_name) in configs.iter() {
            self.generate_kubeconfig(config_name, credential_name)?;
        }

        // Generate kubeconfigs for worker nodes
        for (i, _node) in self.config.worker_nodes.clone().iter().enumerate() {
            let node_name = format!("node-{}", i + 1);
            let credential_name = format!("system:node:{}", node_name);
            self.generate_kubeconfig(&node_name, &credential_name)?;
        }

        // Distribute all kubeconfigs
        self.distribute_kubeconfigs()?;

        self.log("Kubeconfig generation and distribution completed");
        Ok(())
    }

    fn distribute_kubeconfigs(&mut self) -> io::Result<()> {
        self.log("Distributing kubeconfig files...");
        let mut cert_ops = self.create_certificate_operations()?;

        // Distribute admin kubeconfig to control plane
        cert_ops.copy_to_k8s_paths("kubeconfig/admin.conf", &self.config.control_plane)?;

        // Distribute worker kubeconfigs
        for (i, node) in self.config.worker_nodes.iter().enumerate() {
            let node_config = format!("kubeconfig/node-{}.conf", i + 1);
            cert_ops.copy_to_k8s_paths(&node_config, node)?;
        }

        // Distribute controller-manager kubeconfig
        cert_ops.copy_to_k8s_paths(
            "kubeconfig/controller-manager.conf",
            &self.config.control_plane,
        )?;

        // Distribute scheduler kubeconfig
        cert_ops.copy_to_k8s_paths("kubeconfig/scheduler.conf", &self.config.control_plane)?;

        self.log("Kubeconfig distribution completed");
        Ok(())
    }

    fn execute_kubectl_command(&self, args: &[&str]) -> io::Result<()> {
        let output = Command::new("kubectl").args(args).output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "kubectl command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        Ok(())
    }

    pub fn generate_encryption_config(&mut self) -> io::Result<()> {
        self.log("Generating encryption config...");
        let mut cert_ops = self.create_certificate_operations()?;

        // Generate random encryption key
        let encryption_key = self.generate_random_key(32)?;
        let config = self.create_encryption_config(&encryption_key)?;

        // Write config to file
        let config_path = PathBuf::from("encryption-config.yaml");
        fs::write(&config_path, config)?;

        // Distribute to control plane using cert_ops
        cert_ops.copy_to_k8s_paths("encryption-config", &self.config.control_plane)?;

        // // Distribute to control plane
        // cert_ops.copy_with_sudo(
        //     config_path.to_str().unwrap(),
        //     &format!("{}/encryption-config.yaml", self.config.remote_dir),
        //     &self.config.control_plane,
        // )?;

        self.log("Encryption config generated and distributed");
        Ok(())
    }

    fn create_encryption_config(&self, key: &str) -> io::Result<String> {
        Ok(format!(
            r#"kind: EncryptionConfig
apiVersion: v1
resources:
  - resources:
      - secrets
    providers:
      - aescbc:
          keys:
            - name: key1
              secret: {}
      - identity: {{}}"#,
            key
        ))
    }

    fn generate_random_key(&self, length: usize) -> io::Result<String> {
        let output = Command::new("head")
            .args(&["-c", &length.to_string(), "/dev/urandom"])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to generate random key",
            ));
        }

        Ok(general_purpose::STANDARD.encode(&output.stdout))
    }

    pub fn save_config(&self) -> io::Result<()> {
        let config_path = PathBuf::from("cluster_config.json");
        self.config.save_to_file(config_path.to_str().unwrap())
    }

    pub fn get_status_info(&self) -> Vec<Line> {
        vec![
            Line::from(vec![
                Span::styled("Current Operation: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.current_operation,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Control Plane: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.config.control_plane,
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Worker Nodes: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    self.config.worker_nodes.join(", "),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("SSH Key: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.config.ssh_key_path,
                    Style::default().fg(Color::Yellow),
                ),
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
                        // let mut cert_ops = self.create_certificate_operations()?;
                        let control_plane = self.config.control_plane.clone();

                        if let Err(e) = cert_ops.copy_to_k8s_paths("kubernetes", &control_plane) {
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
                        // let cert_ops = self.create_certificate_operations()?;
                        let all_hosts = self.get_all_hosts();
                        let mut success = true;

                        self.cert_tracker.mark_verified("Kubernetes CA", false);
                        for host in &all_hosts {
                            if let Err(e) = cert_ops.copy_to_k8s_paths("kubernetes", host) {
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
                        match self
                            .get_cert_ops()
                            .distribute_certificates("kubernetes-ca", &hosts)
                        {
                            Ok(_) => {
                                self.log(
                                    "CA chain certificates created and distributed successfully",
                                );
                                self.cert_tracker.mark_distributed("ca-chain")
                            }
                            Err(e) => self.log(&format!(
                                "Failed to create and distribute CA chain certificates: {}",
                                e
                            )),
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
                            Err(e) => self.log(&format!("Automation failed: {}", e)),
                        }
                    }
                }

                ConfirmationCallback::DistributePending => {
                    if confirmed {
                        self.mode = AppMode::Normal;
                        let mut cert_ops = self.create_certificate_operations()?;
                        // Get the data we need first
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
                            for (cert_type, _, hosts) in pending_certs {
                                self.log(&format!("Distributing {} certificate...", cert_type));

                                let mut cert_success = true;

                                if cert_type == "root-ca" {
                                    continue;
                                }

                                for host in hosts {
                                    self.log(&format!("{}", host));
                                    match cert_ops.copy_to_k8s_paths(&cert_type, &host) {
                                        Ok(_) => {
                                            self.log(&format!(
                                                "Successfully distributed {} certificate to {}",
                                                cert_type, host
                                            ));
                                        }
                                        Err(e) => {
                                            self.log(&format!(
                                                "Failed to distribute {} certificate to {}: {}",
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

        // let nodes: Vec<(usize, String)> = self
        //     .config
        //     .worker_nodes
        //     .iter()
        //     .enumerate()
        //     .map(|(i, node)| (i, node.clone()))
        //     .collect();

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

    fn create_extensions_file(&self, path: &Path, config: &CertificateConfig) -> io::Result<()> {
        let mut content = String::new();

        if !config.key_usage.is_empty() {
            content.push_str(&format!("keyUsage = {}\n", config.key_usage.join(", ")));
        }

        if !config.extended_key_usage.is_empty() {
            content.push_str(&format!(
                "extendedKeyUsage = {}\n",
                config.extended_key_usage.join(", ")
            ));
        }

        if !config.alt_names.is_empty() {
            content.push_str("subjectAltName = @alt_names\n\n[alt_names]\n");
            for (i, name) in config.alt_names.iter().enumerate() {
                if name.starts_with("IP:") {
                    content.push_str(&format!("IP.{} = {}\n", i + 1, &name[3..]));
                } else if name.starts_with("DNS:") {
                    content.push_str(&format!("DNS.{} = {}\n", i + 1, &name[4..]));
                }
            }
        }

        fs::write(path, content)
    }

    fn generate_worker_kubeconfigs(&mut self) -> io::Result<()> {
        let mut cert_ops = self.create_certificate_operations()?;

        for (i, node) in self.config.worker_nodes.clone().iter().enumerate() {
            let node_name = format!("node-{}", i + 1);
            let credential_name = format!("system:node:{}", node_name);

            // Generate kubeconfig
            self.generate_kubeconfig(&node_name, &credential_name)?;

            // Use cert_ops for distribution
            cert_ops.copy_to_k8s_paths(&format!("kubeconfig/{}", node_name), node)?;
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

    pub fn log(&mut self, message: &str) {
        self.logs.push(format!(
            "{}: {}",
            chrono::Local::now().format("%H:%M:%S"),
            message
        ));

        // Auto-scroll to bottom if not in view mode
        if self.mode != AppMode::ViewLogs {
            self.scroll_to_bottom()
        }
    }

    fn debug_log(&mut self, message: &str) {
        if self.debug {
            self.log(&format!("[DEBUG] {}", message));
        }
    }

    // Helper method to scroll to bottom explicitly
    pub fn scroll_to_bottom(&mut self) {
        let visible_height = 9;
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
