// config/types.rs
use serde::{Deserialize, Serialize};
use std::{fs, io};

use crate::discovery;

#[derive(Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub control_plane: String,
    pub worker_nodes: Vec<String>,
    pub remote_user: String,
    pub ssh_key_path: String,
    pub remote_dir: String,
}

impl ClusterConfig {
    pub fn default() -> Self {
        Self {
            control_plane: "1.2.3.4".to_string(),
            worker_nodes: vec!["1.2.3.4".to_string()],
            remote_user: "adminuser".to_string(),
            remote_dir: "/etc/kubernetes/pki".to_string(),
            ssh_key_path: "~/.ssh/id_rsa".to_string(),
        }
    }

    // pub fn update_control_plane(&mut self, control_plane: String) {
    //     self.control_plane = control_plane;
    // }
    //
    // pub fn update_worker_nodes(&mut self, worker_nodes: Vec<String>) {
    //     self.worker_nodes = worker_nodes;
    // }

    pub async fn load_from_file(path: &str) -> io::Result<Self> {
        let config_str = fs::read_to_string(path)?;
        let mut config: Self = serde_json::from_str(&config_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Discover control plane IP
        match &config.control_plane {
            hostname => match discovery::resolve_hostname(hostname).await {
                Ok(ip) => config.control_plane = ip,
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to resolve control plane hostname: {}", e),
                    ))
                }
            },
        }

        // Discover worker node IPs
        match &config.worker_nodes {
            nodes => {
                let mut resolved_nodes = Vec::new();
                for node in nodes {
                    match discovery::resolve_hostname(node).await {
                        Ok(ip) => resolved_nodes.push(ip),
                        Err(e) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Failed to resolve worker node hostname {}: {}", node, e),
                            ))
                        }
                    }
                }
                config.worker_nodes = resolved_nodes;
            }
        }

        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, config_str)
    }
}
