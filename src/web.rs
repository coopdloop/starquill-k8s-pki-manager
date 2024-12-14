use axum::{
    debug_handler,
    extract::State,
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app::{CertManager, CertStatus, ClusterInfo, ConnectivityStatus, NodeInfo},
    discovery::{self, CertificateInfoSchema, NodeTrustInfoSchema},
    types::{ApiServerMetrics, ControlPlaneMetrics, EtcdMetrics, SchedulerMetrics},
};

#[derive(OpenApi)]
#[openapi(
    paths(cluster_handler, certificates_handler),
    components(schemas(
        ClusterInfo,
        NodeInfo,
        CertStatus,
        ComponentInfo,
        CertificateDetail,
        WorkerNodeInfo,
        NodeMetrics
    ))
)]
struct ApiDoc;

// New structures for API responses
#[derive(Serialize)]
struct ControlPlaneInfo {
    api_server: ComponentInfo,
    etcd: ComponentInfo,
    scheduler: ComponentInfo,
    certificates: Vec<CertificateDetail>,
}

#[derive(Serialize, ToSchema)]
struct ComponentMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_usage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_usage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_latency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_rate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    db_size: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct ComponentInfo {
    version: String,
    status: String,
    uptime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_info: Option<String>,
    metrics: ComponentMetrics,
}

#[derive(Serialize, ToSchema)]
enum Metrics {
    ApiServerMetrics,
    ControlPlaneMetrics,
    EtcMetrics,
}

#[derive(Serialize, ToSchema)]
struct CertificateDetail {
    #[schema(example = "Certificates")]
    name: String,
    expires: String,
    status: String,
    cert_type: String,
    issuer: String,
    nodes: Vec<String>,
}

#[derive(Serialize, ToSchema)]
struct WorkerNodeInfo {
    id: String,
    name: String,
    ip: String,
    status: String,
    metrics: NodeMetrics,
    certificates: Vec<CertificateDetail>,
}

#[derive(Serialize, ToSchema)]
pub struct TrustValidationResponse {
    nodes: HashMap<String, NodeTrustInfoSchema>,
}

#[derive(Serialize, ToSchema)]
struct NodeMetrics {
    cpu: String,
    memory: String,
    disk: String,
}

#[derive(Default)]
pub struct WebServerState {
    pub is_running: bool,
    pub port: u16,
    pub cert_manager: Option<Arc<RwLock<CertManager>>>,
}

impl WebServerState {
    pub fn new(port: Option<u16>) -> Self {
        Self {
            port: port.unwrap_or(3000), // Default to port 3000 if none specified
            is_running: false,
            cert_manager: None,
        }
    }
}

// Helper function to create component metrics
fn create_component_metrics<T: std::fmt::Debug>(_component_metrics: Option<&T>) -> ComponentMetrics {
    // Use type-specific logic if needed
    ComponentMetrics {
        cpu_usage: Some("45%".to_string()),
        memory_usage: Some("60%".to_string()),
        request_latency: Some("10ms".to_string()),
        request_rate: Some("100 req/s".to_string()),
        db_size: None,
    }
}

// Handler for /api/control-plane
async fn control_plane_handler(State(state): State<Arc<RwLock<WebServerState>>>) -> Response {
    let cert_manager = {
        let state_guard = state.read().unwrap();
        match state_guard.cert_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    Json(serde_json::json!({
                        "error": "CertManager not initialized"
                    })),
                )
                    .into_response();
            }
        }
    };

    // Get control plane information
    let manager = cert_manager.read().unwrap();

    let metrics = manager
        .metrics_collector
        .as_ref()
        .and_then(|collector| collector.collect_metrics());

    // Extract certificates for control plane
    let certificates = manager
        .cert_tracker
        .certificates
        .iter()
        .filter(|cert| cert.hosts.contains(&manager.config.control_plane))
        .map(|cert| CertificateDetail {
            name: cert.cert_type.clone(),
            expires: cert.generated.to_rfc3339(),
            status: if cert.distributed.is_some() {
                "Valid".to_string()
            } else {
                "Pending".to_string()
            },
            cert_type: "Server".to_string(), // You might want to derive this from cert properties
            issuer: "Kubernetes CA".to_string(), // You might want to derive this from cert properties
            nodes: cert.hosts.clone(),           // Include the nodes that own this cert
        })
        .collect();

    let info = ControlPlaneInfo {
        api_server: ComponentInfo {
            version: "v1.26.1".to_string(),
            status: "Healthy".to_string(),
            uptime: "15d 4h 23m".to_string(),
            extra_info: None,
            metrics: metrics.as_ref().map_or_else(
                || create_component_metrics::<ApiServerMetrics>(None),
                |m| create_component_metrics::<ApiServerMetrics>(Some(&m.api_server)),
            ),
        },
        etcd: ComponentInfo {
            version: "3.5.6".to_string(),
            status: "Healthy".to_string(),
            uptime: "15d 4h 23m".to_string(),
            extra_info: metrics
                .as_ref()
                .map(|m| m.etcd.db_size.clone())
                .or_else(|| Some("Unknown".to_string())),
            metrics: metrics.as_ref().map_or_else(
                || create_component_metrics::<EtcdMetrics>(None),
                |m| create_component_metrics::<EtcdMetrics>(Some(&m.etcd)),
            ),
        },
        scheduler: ComponentInfo {
            version: "v1.26.1".to_string(),
            status: "Healthy".to_string(),
            uptime: "15d 4h 23m".to_string(),
            extra_info: None,
            metrics: metrics.as_ref().map_or_else(
                || create_component_metrics::<SchedulerMetrics>(None),
                |m| create_component_metrics::<SchedulerMetrics>(Some(&m.scheduler)),
            ),
        },
        certificates,
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({ "data": info })),
    )
        .into_response()
}

// Handler for /api/worker-nodes
#[utoipa::path(
    get,
    path = "/api/worker-nodes",
    responses(
        (status = 200, description = "Get worker nodes", body = Vec<WorkerNodeInfo>)
    )
)]
#[debug_handler]
async fn worker_nodes_handler(State(state): State<Arc<RwLock<WebServerState>>>) -> Response {
    let cert_manager = {
        let state_guard = state.read().unwrap();
        match state_guard.cert_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    Json(serde_json::json!({
                        "error": "CertManager not initialized"
                    })),
                )
                    .into_response();
            }
        }
    };

    let manager = cert_manager.read().unwrap();

    let workers: Vec<WorkerNodeInfo> = manager
        .config
        .worker_nodes
        .iter()
        .enumerate()
        .map(|(i, ip)| {
            let certificates = manager
                .cert_tracker
                .certificates
                .iter()
                .filter(|cert| cert.hosts.contains(ip))
                .map(|cert| CertificateDetail {
                    name: cert.cert_type.clone(),
                    expires: cert.generated.to_rfc3339(),
                    status: if cert.distributed.is_some() {
                        "Valid".to_string()
                    } else {
                        "Pending".to_string()
                    },
                    cert_type: "Client".to_string(),
                    issuer: "Kubernetes CA".to_string(),
                    nodes: cert.hosts.clone(), // Include the nodes that own this cert
                })
                .collect();

            WorkerNodeInfo {
                id: format!("worker{}", i + 1),
                name: format!("Worker {}", i + 1),
                ip: ip.clone(),
                status: "Ready".to_string(),
                metrics: NodeMetrics {
                    cpu: "45%".to_string(),
                    memory: "60%".to_string(),
                    disk: "32%".to_string(),
                },
                certificates,
            }
        })
        .collect();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({ "data": workers })),
    )
        .into_response()
}

// Handler for /api/certificates
#[utoipa::path(
    get,
    path = "/api/certificates",
    responses(
        (status = 200, description = "Get certificate information", body = Vec<CertificateDetail>)
    )
)]
#[debug_handler]
async fn certificates_handler(State(state): State<Arc<RwLock<WebServerState>>>) -> Response {
    let cert_manager = {
        let state_guard = state.read().unwrap();
        match state_guard.cert_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    Json(serde_json::json!({
                        "error": "CertManager not initialized"
                    })),
                )
                    .into_response();
            }
        }
    };

    let manager = cert_manager.read().unwrap();

    let certificates: Vec<CertificateDetail> = manager
        .cert_tracker
        .certificates
        .iter()
        .map(|cert| CertificateDetail {
            name: cert.cert_type.clone(),
            expires: cert.generated.to_rfc3339(),
            status: if cert.distributed.is_some() {
                "Valid".to_string()
            } else {
                "Warning".to_string()
            },
            cert_type: if cert.cert_type.contains("client") {
                "Client".to_string()
            } else if cert.cert_type.contains("server") {
                "Server".to_string()
            } else {
                "Peer".to_string()
            },
            issuer: "Kubernetes CA".to_string(),
            nodes: cert.hosts.clone(), // Include the nodes that own this cert
        })
        .collect();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({ "data": certificates })),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/api/cluster",
    responses(
        (status = 200, description = "Get cluster information", body = ClusterInfo)
    )
)]
#[debug_handler]
async fn cluster_handler(State(state): State<Arc<RwLock<WebServerState>>>) -> Response {
    // Get CertManager reference with minimal lock time
    let cert_manager = {
        let state_guard = state.read().unwrap();
        match state_guard.cert_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    Json(serde_json::json!({
                        "error": "CertManager not initialized"
                    })),
                )
                    .into_response();
            }
        }
    };

    let manager = cert_manager.read().unwrap();

    let ssh_cache = discovery::SSHConnectionCache::load().unwrap_or_default();

    // Get all nodes
    let all_nodes: Vec<String> = vec![manager.config.control_plane.clone()]
        .into_iter()
        .chain(manager.config.worker_nodes.clone())
        .collect();

    // Get unreachable nodes
    let unreachable_nodes: Vec<String> = all_nodes
        .iter()
        .filter(|node| !ssh_cache.is_verified(node))
        .cloned()
        .collect();

    let available_nodes = all_nodes.len() - unreachable_nodes.len();

    let info = ClusterInfo {
        control_plane: NodeInfo {
            ip: manager.config.control_plane.clone(),
            certs: manager
                .cert_tracker
                .certificates
                .iter()
                .filter(|c| c.hosts.contains(&manager.config.control_plane)) // Only include certs for this node
                .filter(|c| !c.cert_type.contains("root-ca")) // Ignore root ca when getting certs
                .map(|c| CertStatus {
                    cert_type: c.cert_type.clone(),
                    status: if c.distributed.is_some() {
                        "Distributed".into()
                    } else {
                        "Generated".into()
                    },
                    last_updated: c
                        .distributed
                        .or(Some(c.generated))
                        .map(|dt| dt.to_rfc3339()),
                })
                .collect(),
        },
        workers: manager
            .config
            .worker_nodes
            .iter()
            .map(|ip| NodeInfo {
                ip: ip.clone(),
                certs: manager
                    .cert_tracker
                    .certificates
                    .iter()
                    .filter(|c| c.hosts.contains(ip))
                    .filter(|c| !c.cert_type.contains("root-ca")) // Ignore root ca when getting certs
                    .map(|c| CertStatus {
                        cert_type: c.cert_type.clone(),
                        status: if c.distributed.is_some() {
                            "Distributed".into()
                        } else {
                            "Generated".into()
                        },
                        last_updated: c
                            .distributed
                            .or(Some(c.generated))
                            .map(|dt| dt.to_rfc3339()),
                    })
                    .collect(),
            })
            .collect(),
        connectivity: ConnectivityStatus {
            unreachable_nodes: unreachable_nodes.clone(),
            last_checked: chrono::Utc::now().to_rfc3339(),
            total_nodes: all_nodes.len(),
            available_nodes,
        },
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({ "data": info })),
    )
        .into_response()
}

// New debug handler for certificates
async fn debug_certificates(State(state): State<Arc<RwLock<WebServerState>>>) -> Response {
    let cert_manager = {
        let state_guard = state.read().unwrap();
        match state_guard.cert_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    Json(serde_json::json!({
                        "error": "CertManager not initialized"
                    })),
                )
                    .into_response();
            }
        }
    };

    let manager = cert_manager.read().unwrap();

    let debug_info = manager
        .cert_tracker
        .certificates
        .iter()
        .map(|cert| {
            serde_json::json!({
                "cert_type": cert.cert_type,
                "hosts": cert.hosts,
                "distributed": cert.distributed.is_some(),
                "generated": cert.generated
            })
        })
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({
            "total_certificates": manager.cert_tracker.certificates.len(),
            "certificates": debug_info
        })),
    )
        .into_response()
}

// Add documentation for trust validation endpoint
#[utoipa::path(
    get,
    path = "/api/trust-validate",
    responses(
        (status = 200, description = "Get trust validation information", body = TrustValidationResponse)
    )
)]
// Add TrustValidationResponse to schema components
#[debug_handler]
async fn trust_validation_handler(State(state): State<Arc<RwLock<WebServerState>>>) -> Response {
    let cert_manager = {
        let state_guard = state.read().unwrap();
        match state_guard.cert_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    Json(serde_json::json!({
                        "error": "CertManager not initialized"
                    })),
                )
                    .into_response();
            }
        }
    };

    let manager = cert_manager.read().unwrap();

    // Use the local trust_store if it exists
    let trust_store = manager.trust_store.clone().unwrap_or_default();

    // Convert to schema-friendly format
    let converted_store: HashMap<String, NodeTrustInfoSchema> = trust_store
        .into_iter()
        .map(|(k, v)| {
            let cert_schemas: Vec<CertificateInfoSchema> = v
                .certificates
                .into_iter()
                .map(|cert| CertificateInfoSchema {
                    path: cert.path.to_string_lossy().to_string(),
                    subject: cert.subject.clone(),
                    issuer: cert.issuer.clone(),
                    not_before: cert.not_before.to_rfc3339(),
                    not_after: cert.not_after.to_rfc3339(),
                    serial: cert.serial.clone(),
                    fingerprint: cert.fingerprint.clone(),
                    is_ca: cert.is_ca,
                    last_verified: cert.last_verified.map(|dt| dt.to_rfc3339()),
                    verification_error: cert.verification_error.clone(),
                })
                .collect();

            (
                k,
                NodeTrustInfoSchema {
                    node_ip: v.node_ip.clone(),
                    certificates: cert_schemas,
                    trust_chain_valid: v.trust_chain_valid,
                    permissions_valid: v.permissions_valid,
                    expiring_soon: v.expiring_soon.clone(),
                    last_checked: v.last_checked.to_rfc3339(),
                },
            )
        })
        .collect();

    let response = TrustValidationResponse {
        nodes: converted_store,
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({
            "data": response
        })),
    )
        .into_response()
}

pub async fn start_web_server(
    state: Arc<RwLock<WebServerState>>,
    shutdown: tokio::sync::oneshot::Receiver<()>,
) {
    let port = {
        let state = state.read().unwrap();
        state.port
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/cluster", get(cluster_handler))
        .route("/api/control-plane", get(control_plane_handler))
        .route("/api/worker-nodes", get(worker_nodes_handler))
        .route("/api/certificates", get(certificates_handler))
        .route("/api/debug/certificates", get(debug_certificates))
        .route("/api/trust-validate", get(trust_validation_handler))
        .nest_service(
            "/",
            get_service(
                ServeDir::new("webapp/dist").fallback(ServeFile::new("webapp/dist/index.html")),
            ),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(state.clone());

    let addr = format!("0.0.0.0:{}", port);
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            if let Some(ref cm) = state.read().unwrap().cert_manager.as_ref() {
                let mut manager = cm.write().unwrap();
                manager.log(&format!("Web server listening on {}", addr));
                // manager.log("Available endpoints:");
                // manager.log("  - /health");
                // manager.log("  - /api/cluster");
                // manager.log("  - /swagger-ui");
            }

            {
                let mut state = state.write().unwrap();
                state.is_running = true;
            }

            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown.await.ok();
                })
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Server error: {}", e);
                    if let Some(ref cm) = state.read().unwrap().cert_manager.as_ref() {
                        let mut manager = cm.write().unwrap();
                        manager.log(&format!("Server error: {}", e));
                    }
                });
        }
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", addr, e);
            if let Some(ref cm) = state.read().unwrap().cert_manager.as_ref() {
                let mut manager = cm.write().unwrap();
                manager.log(&format!("Failed to bind to address {}: {}", addr, e));
            }
        }
    }
}

async fn health_check() -> &'static str {
    "OK"
}
