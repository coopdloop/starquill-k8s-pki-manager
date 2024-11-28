# Starquill - Kubernetes Certificate Management Suite

Starquill is a comprehensive Kubernetes certificate management suite that combines a powerful Terminal User Interface (TUI) with a modern web application for visual cluster management. Built with Rust and React, it provides a robust solution for generating, distributing, and managing certificates across your Kubernetes cluster.


![Starquill TUI](https://raw.githubusercontent.com/coopdloop/starquill-k8s-pki-manager/main/docs/images/starquill_web_ui.png)

## Features

### Core Certificate Management
- Complete certificate lifecycle management:
  - Root CA and Kubernetes CA generation
  - API Server certificates
  - Node certificates
  - Service Account keys
  - Controller Manager and Scheduler certificates
- Automated certificate chain creation and validation
- Secure certificate distribution via SSH
- Encryption configuration for data-at-rest

### Terminal User Interface (TUI)
- Interactive terminal dashboard built with Ratatui
- Real-time operation logging
- Visual certificate status tracking
- Interactive configuration management
- Operation confirmation dialogs

### Web Interface & API
- Modern React/Vite web application
- Visual drag-and-drop node topology editor
- Real-time cluster status updates
- REST API with Swagger documentation
- CORS support for cross-origin requests

### API Endpoints
- `/api/cluster` - Get cluster information and certificate status
- `/health` - Server health check endpoint
- `/swagger-ui` - Interactive API documentation
- Static file serving for web application

## Architecture

### Backend (Rust)
- Built with Axum web framework
- Async runtime with Tokio
- Thread-safe state management using Arc and RwLock
- OpenAPI documentation using Utoipa
- Graceful shutdown support
- Structured logging system

### Frontend (React/Vite)
- Single-page application
- Real-time cluster visualization
- Certificate status dashboard
- Node management interface
- Responsive design

## Installation

### Prerequisites
- Rust 1.70 or higher
- Node.js 16+ and npm
- OpenSSL development libraries
- SSH access to cluster nodes

### Building
```bash
# Build backend
cargo build --release

# Build frontend
cd webapp
npm install
npm run build
```

## Configuration

### Server Configuration
- Default port: 3000 (configurable)
- CORS configured for cross-origin requests
- Supports API documentation via Swagger UI

### Cluster Configuration
- Control plane node settings
- Worker node management
- SSH key configuration
- Remote directory structure
- Certificate distribution paths

## Usage

### Starting the Application
```bash
# Start with default settings
./starquill

# Specify custom port
./starquill --port 8080

# TUI-only mode
./starquill --no-web
```

### Web Interface
Access the web interface at `http://localhost:3000`
- View cluster topology
- Monitor certificate status
- Manage node configuration
- Track certificate distribution

### API Documentation
Access Swagger UI at `http://localhost:3000/swagger-ui`
- Interactive API documentation
- Request/response examples
- API schema information

### Certificate Operations
1. Configure cluster details via TUI or web interface
2. Generate Root CA and Kubernetes CA
3. Generate component certificates
4. Distribute certificates securely
5. Verify distribution and trust chain
6. Monitor status through web interface

## Security

### Certificate Security
- Secure generation and storage
- Proper file permissions
- SSH-based secure distribution
- Certificate chain verification

### Web Security
- CORS protection
- Secure state management
- Input validation
- Error handling

## Development

### Backend Development
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test
```

### Frontend Development
```bash
cd webapp
npm run dev
```

## API Reference

### GET /api/cluster
Returns cluster information including:
- Control plane details
- Worker node information
- Certificate status for each node
- Distribution status
- Last update timestamps

### Response Format
```json
{
  "data": {
    "control_plane": {
      "ip": "string",
      "certs": [
        {
          "cert_type": "string",
          "status": "string",
          "last_updated": "string"
        }
      ]
    },
    "workers": [
      {
        "ip": "string",
        "certs": [...]
      }
    ]
  }
}
```

## Contributing
Contributions welcome! Please read our contributing guidelines and submit PRs to my repository.

## License

(Still in early development)

[Insert License Information]

## Support
For issues and feature requests, please use our GitHub issue tracker.
