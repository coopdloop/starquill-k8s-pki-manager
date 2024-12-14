// src/discovery/kubeconfig.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KubeConfig {
    pub clusters: Vec<ClusterConfig>,
    pub users: Vec<UserConfig>,
    pub contexts: Vec<ContextConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ClusterConfig {
    pub name: String,
    pub server: String,
    pub certificate_authority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub client_certificate: Option<String>,
    pub client_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContextConfig {
    pub name: String,
    pub cluster: String,
    pub user: String,
}
