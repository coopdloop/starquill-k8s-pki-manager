use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub struct KubeConfigGenerator {
    control_plane_ip: String,
    output_dir: PathBuf,
    ca_path: PathBuf,
}

impl KubeConfigGenerator {
    pub fn new(control_plane_ip: String, output_dir: PathBuf, ca_path: PathBuf) -> Self {
        Self {
            control_plane_ip,
            output_dir,
            ca_path,
        }
    }

    pub fn generate_all_kubeconfigs(&self) -> io::Result<()> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;

        // Generate admin kubeconfig
        self.generate_kubeconfig("admin", "default-admin")?;

        // Generate controller-manager kubeconfig
        self.generate_kubeconfig(
            "controller-manager",
            "system:kube-controller-manager",
        )?;

        // Generate scheduler kubeconfig
        self.generate_kubeconfig("scheduler", "system:kube-scheduler")?;

        Ok(())
    }

    pub fn generate_node_kubeconfigs(&self, node_indices: &[(usize, String)]) -> io::Result<()> {
        for (i, _) in node_indices {
            let node_name = format!("node-{}", i + 1);
            let credential_name = format!("system:node:worker-node-{}", i + 1);
            self.generate_kubeconfig(&node_name, &credential_name)?;
        }
        Ok(())
    }

    pub fn generate_kubeconfig(&self, config_name: &str, credential_name: &str) -> io::Result<()> {
        let kubeconfig_path = self.output_dir.join(format!("{}.conf", config_name));
        let api_server = format!("https://{}:6443", self.control_plane_ip);

        // Set cluster
        Command::new("kubectl")
            .args(&[
                "config",
                "set-cluster",
                "default-cluster",
                &format!("--kubeconfig={}", kubeconfig_path.display()),
                &format!("--server={}", api_server),
                &format!("--certificate-authority={}", self.ca_path.display()),
                "--embed-certs=true",
            ])
            .output()?;

        // Set credentials
        Command::new("kubectl")
            .args(&[
                "config",
                "set-credentials",
                credential_name,
                &format!("--kubeconfig={}", kubeconfig_path.display()),
                &format!("--client-certificate={}/{}.crt", config_name, config_name),
                &format!("--client-key={}/{}.key", config_name, config_name),
                "--embed-certs=true",
            ])
            .output()?;

        // Set context
        Command::new("kubectl")
            .args(&[
                "config",
                "set-context",
                "default-system",
                &format!("--kubeconfig={}", kubeconfig_path.display()),
                "--cluster=default-cluster",
                &format!("--user={}", credential_name),
            ])
            .output()?;

        // Use context
        Command::new("kubectl")
            .args(&[
                "config",
                "use-context",
                "default-system",
                &format!("--kubeconfig={}", kubeconfig_path.display()),
            ])
            .output()?;

        Ok(())
    }
}
