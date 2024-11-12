// config/types.rs
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

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
            control_plane: "52.182.169.94".to_string(),
            worker_nodes: vec!["52.182.169.95".to_string(), "52.182.169.100".to_string()],
            remote_user: "adminuser".to_string(),
            remote_dir: "/etc/kubernetes/pki".to_string(),
            ssh_key_path: "~/.ssh/id_rsa".to_string(),
        }
    }

    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let config_str = fs::read_to_string(path)?;
        serde_json::from_str(&config_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, config_str)
    }

    pub fn validate(&self) -> io::Result<()> {
        if !Path::new(&shellexpand::tilde(&self.ssh_key_path).to_string()).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("SSH key not found at: {}", self.ssh_key_path),
            ));
        }
        Ok(())
    }
}

