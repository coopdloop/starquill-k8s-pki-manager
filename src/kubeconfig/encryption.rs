use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize)]
struct EncryptionConfig {
    kind: String,
    api_version: String,
    resources: Vec<ResourceConfig>,
}

#[derive(Serialize)]
struct ResourceConfig {
    resources: Vec<String>,
    providers: Vec<Provider>,
}

#[derive(Serialize)]
struct Provider {
    #[serde(skip_serializing_if = "Option::is_none")]
    aescbc: Option<AESCBCConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity: Option<IdentityConfig>,
}

#[derive(Serialize)]
struct AESCBCConfig {
    keys: Vec<Key>,
}

#[derive(Serialize)]
struct Key {
    name: String,
    secret: String,
}

#[derive(Serialize)]
struct IdentityConfig {}

pub struct EncryptionConfigGenerator {
    output_path: PathBuf,
}

impl EncryptionConfigGenerator {
    pub fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    pub fn generate_config(&self) -> io::Result<()> {
        let encryption_key = self.generate_random_key(32)?;

        let config = EncryptionConfig {
            kind: "EncryptionConfig".to_string(),
            api_version: "v1".to_string(),
            resources: vec![ResourceConfig {
                resources: vec!["secrets".to_string()],
                providers: vec![
                    Provider {
                        aescbc: Some(AESCBCConfig {
                            keys: vec![Key {
                                name: "key1".to_string(),
                                secret: encryption_key,
                            }],
                        }),
                        identity: None,
                    },
                    Provider {
                        aescbc: None,
                        identity: Some(IdentityConfig {}),
                    },
                ],
            }],
        };

        let yaml = serde_yaml::to_string(&config)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(&self.output_path, yaml)?;

        Ok(())
    }

    fn generate_random_key(&self, length: usize) -> io::Result<String> {
        let output = Command::new("head")
            .args(&["-c", &length.to_string(), "/dev/urandom"])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to generate random key"));
        }

        Ok(general_purpose::STANDARD.encode(&output.stdout))
    }
}
