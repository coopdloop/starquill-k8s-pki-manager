// src/ssh.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::Command;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time;

const CACHE_FILE: &str = "ssh_cache.json";
const CACHE_VALIDITY_DURATION: u64 = 300; // 5 minutes in seconds
const RECHECK_INTERVAL: u64 = 30; // 5 minutes in seconds

#[derive(Serialize, Deserialize, Default)]
pub struct SSHConnectionCache {
    connections: HashMap<String, ConnectionStatus>,
}

#[derive(Serialize, Deserialize)]
struct ConnectionStatus {
    verified: bool,
    timestamp: u64,
}

impl SSHConnectionCache {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn load() -> io::Result<Self> {
        match fs::read_to_string(CACHE_FILE) {
            Ok(contents) => serde_json::from_str(&contents).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse cache: {}", e),
                )
            }),
            Err(_) => Ok(Self::new()),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let contents = serde_json::to_string(self)?;
        fs::write(CACHE_FILE, contents)
    }

    pub fn is_verified(&self, host: &str) -> bool {
        if let Some(status) = self.connections.get(host) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            status.verified && (current_time - status.timestamp) < CACHE_VALIDITY_DURATION
        } else {
            false
        }
    }

    pub fn update_status(&mut self, host: &str, verified: bool) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.connections.insert(
            host.to_string(),
            ConnectionStatus {
                verified,
                timestamp,
            },
        );
    }

    pub fn get_all_hosts(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }

    pub fn needs_recheck(&self, host: &str) -> bool {
        if let Some(status) = self.connections.get(host) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            (current_time - status.timestamp) >= CACHE_VALIDITY_DURATION
        } else {
            true
        }
    }
}

pub async fn verify_ssh_connection(
    host: &str,
    user: &str,
    key_path: &str,
    cache: &mut SSHConnectionCache,
) -> io::Result<bool> {
    // Check if we need to recheck
    if !cache.needs_recheck(host) && cache.is_verified(host) {
        return Ok(true);
    }

    let ssh_command = Command::new("ssh")
        .args([
            "-i",
            key_path,
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "StrictHostKeyChecking=no",
            &format!("{}@{}", user, host),
            "echo 'Connected successfully'",
        ])
        .output()?;

    let success = ssh_command.status.success();

    cache.update_status(host, success);
    cache.save()?;
    Ok(success)
}

use tokio::sync::mpsc;

#[derive(Debug)]
enum CheckMessage {
    Check(String),
    UpdateStatus(String, bool),
}

pub fn start_periodic_check(
    cache: Arc<RwLock<SSHConnectionCache>>,
    user: String,
    key_path: String,
) {
    let (tx, mut rx) = mpsc::channel(32);
    let tx_clone = tx.clone();

    // Clone Arc for the checker task
    let checker_cache = Arc::clone(&cache);

    // Spawn checker task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                CheckMessage::Check(host) => {
                    let success = Command::new("ssh")
                        .args([
                            "-i",
                            &key_path,
                            "-o",
                            "BatchMode=yes",
                            "-o",
                            "ConnectTimeout=5",
                            "-o",
                            "StrictHostKeyChecking=no",
                            &format!("{}@{}", user, host),
                            "echo 'Connected successfully'",
                        ])
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false);

                    let _ = tx.send(CheckMessage::UpdateStatus(host, success)).await;
                }
                CheckMessage::UpdateStatus(host, status) => {
                    if let Ok(mut cache) = checker_cache.write() {
                        cache.update_status(&host, status);
                        // Clear expired entries while we have write lock
                        clear_expired_entries(&mut cache);
                        let _ = cache.save();
                    }
                }
            }
        }
    });

    // Spawn timer task with original cache
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(RECHECK_INTERVAL));

        loop {
            interval.tick().await;

            // Get hosts that need checking
            let hosts_to_check: Vec<String> = {
                let cache_read = cache.read().unwrap();
                cache_read
                    .get_all_hosts()
                    .into_iter()
                    .filter(|host| cache_read.needs_recheck(host))
                    .collect()
            };

            // Send check messages
            for host in hosts_to_check {
                let _ = tx_clone.send(CheckMessage::Check(host)).await;
            }
        }
    });
}

// Helper function to clear expired cache entries
pub fn clear_expired_entries(cache: &mut SSHConnectionCache) {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    cache
        .connections
        .retain(|_, status| (current_time - status.timestamp) < CACHE_VALIDITY_DURATION);
}
