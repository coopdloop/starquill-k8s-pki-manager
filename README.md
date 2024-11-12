# ⭐ Starquill

A powerful TUI-based Kubernetes certificate management tool that simplifies the process of generating, distributing, and managing certificates for Kubernetes clusters.

![Starquill TUI](https://raw.githubusercontent.com/username/starquill/main/docs/images/starquill-tui.png)

## 🌟 Features

- **Interactive TUI Interface**: Built with Ratatui for a seamless terminal user experience
- **Comprehensive Certificate Management**:
  - Root CA generation
  - Kubernetes CA generation
  - API Server certificates
  - Node certificates
  - Service Account key pairs
  - Controller Manager certificates
- **Automated Workflows**: One-click automation for generating and distributing all required certificates
- **Real-time Certificate Status**: Visual tracking of certificate generation and distribution status
- **SSH-based Distribution**: Secure certificate distribution to cluster nodes
- **Certificate Verification**: Built-in verification of generated certificates
- **Configuration Management**: Interactive configuration editor with SSH key path auto-completion

## 🚀 Quick Start

```bash
# Install using cargo
cargo install starquill

# Run with default configuration
starquill

# Run with custom config file
starquill --config cluster-config.yaml

# Run in debug mode
starquill --debug
```

## 📋 Configuration

Create a `config.yaml` file:

```yaml
remote_user: kube-admin
control_plane: 192.168.1.100
worker_nodes:
  - 192.168.1.101
  - 192.168.1.102
remote_dir: /etc/kubernetes/pki
ssh_key_path: ~/.ssh/id_rsa
```

## 🎮 Usage

Navigate the TUI using:
- `↑`/`↓`: Navigate menu items
- `Enter`: Select menu item
- `L`: View logs
- `Q`: Quit application
- `PgUp`/`PgDn`: Scroll logs
- `Esc`: Exit current view

## 🔧 Certificate Operations

Starquill manages the following certificate operations:

1. **Root CA**
   - Generate root certificate authority
   - Establish trust anchor for cluster

2. **Kubernetes CA**
   - Generate intermediate CA
   - Create certificate chain

3. **Control Plane Certificates**
   - API Server certificate
   - Controller Manager certificate
   - Scheduler certificate
   - Service Account key pairs

4. **Node Certificates**
   - Kubelet client certificates
   - Kubelet serving certificates

## 🛠️ Development

```bash
# Clone repository
git clone https://github.com/coopdloop/starquill-k8s-pki-manager

# Build project
cargo build

# Run tests
cargo test

# Run with debug logging
cargo run -- --debug
```

### Project Structure

```
src/
├── app/        # Application logic and state management
├── cert/       # Certificate generation and operations
├── config/     # Configuration handling
├── types/      # Type definitions
├── ui/         # TUI components and rendering
└── utils/      # Utility functions and helpers
```

## 🔐 Security Considerations

- All private keys are generated with appropriate permissions (600)
- SSH-based secure distribution of certificates
- Certificate verification before distribution
- Support for custom certificate validity periods
- Automated cleanup of sensitive temporary files

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Ratatui](https://github.com/tui-rs-revival/ratatui)
- Inspired by the need for simpler Kubernetes certificate management

## 📞 Support

For support, please open an issue in the GitHub repository or contact the maintainers.

---

Built with ❤️ for the Kubernetes community
