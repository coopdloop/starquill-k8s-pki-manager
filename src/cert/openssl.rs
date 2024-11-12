// src/cert/openssl.rs
use super::types::CertificateConfig;
use crate::cert::CertificateType;
use crate::utils::logging::Logger;
use std::{fs, io, path::Path, process::Command};

#[derive(Debug)]
pub struct OpenSSLError {
    pub message: String,
    pub stdout: String,
    pub stderr: String,
}

impl std::fmt::Display for OpenSSLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OpenSSLError {}

pub fn generate_private_key(path: &str, key_size: u32, logger: &mut dyn Logger) -> io::Result<()> {
    logger.debug_log(&format!("Generating private key: {}", path));

    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let output = Command::new("openssl")
        .args(&["genrsa", "-out", path, &key_size.to_string()])
        .output()?;

    if !output.status.success() {
        let error = OpenSSLError {
            message: format!("Failed to generate private key: {}", path),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };
        logger.log(&error.message);
        logger.debug_log(&format!(
            "stdout: {}\nstderr: {}",
            error.stdout, error.stderr
        ));
        return Err(io::Error::new(io::ErrorKind::Other, error.message));
    }

    // Set proper permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }

    logger.debug_log(&format!("Successfully generated private key: {}", path));
    Ok(())
}

pub fn generate_csr(
    config: &CertificateConfig,
    key_path: &str,
    csr_path: &str,
    logger: &mut dyn Logger,
) -> io::Result<()> {
    logger.debug_log(&format!("Generating CSR: {}", csr_path));

    // Validate key exists
    if !Path::new(key_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Private key not found: {}", key_path),
        ));
    }

    let config_content = generate_csr_config(config)?;
    let config_path = format!("{}.cnf", csr_path);
    fs::write(&config_path, config_content)?;

    logger.debug_log(&format!("Using OpenSSL config: {}", config_path));

    let output = Command::new("openssl")
        .args(&[
            "req",
            "-new",
            "-key",
            key_path,
            "-out",
            csr_path,
            "-config",
            &config_path,
            "-batch",
        ])
        .output()?;

    // Clean up config file
    let _ = fs::remove_file(&config_path);

    if !output.status.success() {
        let error = OpenSSLError {
            message: format!("Failed to generate CSR: {}", csr_path),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };
        logger.log(&error.message);
        logger.debug_log(&format!(
            "stdout: {}\nstderr: {}",
            error.stdout, error.stderr
        ));
        return Err(io::Error::new(io::ErrorKind::Other, error.message));
    }

    logger.debug_log(&format!("Successfully generated CSR: {}", csr_path));
    Ok(())
}

pub fn sign_certificate(
    csr_path: &str,
    cert_path: &str,
    ca_cert: &str,
    ca_key: &str,
    config: &CertificateConfig,
    logger: &mut dyn Logger,
) -> io::Result<()> {
    // Check if CA files exist when needed
    if config.cert_type != CertificateType::RootCA {
        if !Path::new(ca_cert).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("CA certificate not found: {}", ca_cert),
            ));
        }
        if !Path::new(ca_key).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("CA key not found: {}", ca_key),
            ));
        }
    }

    logger.debug_log(&format!("Signing certificate: {}", cert_path));

    // Create extensions file
    let extensions_file = format!("{}.ext", cert_path);
    create_extensions_file(&extensions_file, config)?;

    let mut cmd = Command::new("openssl");
    cmd.arg("x509").arg("-req");

    // Build command based on whether this is a self-signed cert
    if config.cert_type == CertificateType::RootCA {
        cmd.args(&["-signkey", ca_key]);
    } else {
        cmd.args(&["-CA", ca_cert, "-CAkey", ca_key, "-CAcreateserial"]);
    }

    cmd.args(&[
        "-in",
        csr_path,
        "-out",
        cert_path,
        "-days",
        &config.validity_days.to_string(),
        "-extfile",
        &extensions_file,
    ]);

    logger.debug_log(&format!("Executing OpenSSL command: {:?}", cmd));

    let output = cmd.output()?;

    // Clean up extensions file
    let _ = fs::remove_file(&extensions_file);

    if !output.status.success() {
        let error = OpenSSLError {
            message: (cert_path.to_string()),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };
        // logger.log(&error.message);
        return Err(io::Error::new(io::ErrorKind::Other, error.message));
    }

    logger.debug_log(&format!("Successfully signed certificate: {}", cert_path));
    Ok(())
}

pub fn verify_certificate(
    cert_path: &str,
    ca_cert: Option<&str>,
    logger: &mut dyn Logger,
) -> io::Result<()> {
    logger.debug_log(&format!("Verifying certificate: {}", cert_path));

    // Basic certificate info check
    let basic_check = Command::new("openssl")
        .args(&["x509", "-in", cert_path, "-noout", "-text"])
        .output()?;

    if !basic_check.status.success() {
        let error = OpenSSLError {
            message: format!("Certificate basic check failed: {}", cert_path),
            stdout: String::from_utf8_lossy(&basic_check.stdout).to_string(),
            stderr: String::from_utf8_lossy(&basic_check.stderr).to_string(),
        };
        logger.log(&error.message);
        logger.debug_log(&format!(
            "stdout: {}\nstderr: {}",
            error.stdout, error.stderr
        ));
        return Err(io::Error::new(io::ErrorKind::Other, error.message));
    }

    // Verify against CA if provided
    if let Some(ca) = ca_cert {
        logger.debug_log(&format!("Verifying against CA: {}", ca));
        let chain_check = Command::new("openssl")
            .args(&["verify", "-CAfile", ca, cert_path])
            .output()?;

        if !chain_check.status.success() {
            let error = OpenSSLError {
                message: format!("Certificate chain verification failed: {}", cert_path),
                stdout: String::from_utf8_lossy(&chain_check.stdout).to_string(),
                stderr: String::from_utf8_lossy(&chain_check.stderr).to_string(),
            };
            logger.log(&error.message);
            logger.debug_log(&format!(
                "stdout: {}\nstderr: {}",
                error.stdout, error.stderr
            ));
            return Err(io::Error::new(io::ErrorKind::Other, error.message));
        }
    }

    logger.debug_log(&format!("Certificate verified successfully: {}", cert_path));
    Ok(())
}

fn generate_csr_config(config: &CertificateConfig) -> io::Result<String> {
    let mut content = format!(
        r#"[req]
req_extensions = v3_req
distinguished_name = req_distinguished_name
prompt = no

[req_distinguished_name]
CN = {}
O = {}

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
"#,
        config.common_name,
        config.organization.as_deref().unwrap_or("Kubernetes")
    );

    if !config.alt_names.is_empty() {
        content.push_str("subjectAltName = @alt_names\n\n[alt_names]\n");
        for (i, name) in config.alt_names.iter().enumerate() {
            if name.starts_with("IP:") {
                content.push_str(&format!("IP.{} = {}\n", i + 1, &name[3..]));
            } else {
                content.push_str(&format!("DNS.{} = {}\n", i + 1, name));
            }
        }
    }

    Ok(content)
}

fn create_extensions_file(path: &str, config: &CertificateConfig) -> io::Result<()> {
    let mut content = String::new();

    // Basic constraints
    match config.cert_type {
        CertificateType::RootCA | CertificateType::KubernetesCA => {
            content.push_str("basicConstraints = critical,CA:TRUE\n");
        }
        _ => {
            content.push_str("basicConstraints = critical,CA:FALSE\n");
        }
    }

    // Key usage
    if !config.key_usage.is_empty() {
        content.push_str(&format!("keyUsage = {}\n", config.key_usage.join(", ")));
    }

    // Extended key usage
    if !config.extended_key_usage.is_empty() {
        content.push_str(&format!(
            "extendedKeyUsage = {}\n",
            config.extended_key_usage.join(", ")
        ));
    }

    // Subject Alternative Names
    if !config.alt_names.is_empty() {
        content.push_str("subjectAltName = @alt_names\n\n[alt_names]\n");
        for (i, name) in config.alt_names.iter().enumerate() {
            if name.starts_with("IP:") {
                content.push_str(&format!("IP.{} = {}\n", i + 1, &name[3..]));
            } else {
                content.push_str(&format!("DNS.{} = {}\n", i + 1, name));
            }
        }
    }

    fs::write(path, content)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::TempDir;
//
//     struct MockLogger {
//         logs: Vec<String>,
//     }
//
//     impl MockLogger {
//         fn new() -> Self {
//             Self { logs: Vec::new() }
//         }
//     }
//
//     impl Logger for MockLogger {
//         fn log(&mut self, message: &str) {
//             self.logs.push(message.to_string());
//         }
//
//         fn debug_log(&mut self, message: &str) {
//             self.logs.push(format!("DEBUG: {}", message));
//         }
//     }
//
//     #[test]
//     fn test_generate_private_key() -> io::Result<()> {
//         let temp_dir = TempDir::new()?;
//         let key_path = temp_dir.path().join("test.key");
//         let mut logger = MockLogger::new();
//
//         generate_private_key(key_path.to_str().unwrap(), 2048, &mut logger)?;
//
//         assert!(key_path.exists());
//         Ok(())
//     }
//
//     // Add more tests...
// }
