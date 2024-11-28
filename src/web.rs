use axum::{
    debug_handler,
    extract::State,
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use std::sync::{Arc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app::{CertManager, CertStatus, ClusterInfo, NodeInfo},
    track_lock_count,
};

#[derive(OpenApi)]
#[openapi(
    paths(cluster_handler),
    components(schemas(ClusterInfo, NodeInfo, CertStatus))
)]
struct ApiDoc;

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
    pub fn set_cert_manager(&mut self, cert_manager: Option<Arc<RwLock<CertManager>>>) {
        track_lock_count(1, "WebServerState:set_cert_manager");
        self.cert_manager = cert_manager;
        track_lock_count(-1, "WebServerState:set_cert_manager_end");
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn set_running(&mut self, running: bool) {
        track_lock_count(1, "WebServerState:set_running");
        self.is_running = running;
        track_lock_count(-1, "WebServerState:set_running_end");
    }
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

    // Get config and certificates with minimal lock time
    let info = {
        let manager = cert_manager.read().unwrap();
        ClusterInfo {
            control_plane: NodeInfo {
                ip: manager.config.control_plane.clone(),
                certs: manager
                    .cert_tracker
                    .certificates
                    .iter()
                    .filter(|c| c.hosts.contains(&manager.config.control_plane))
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
        }
    };

    // Log success (minimal lock time)
    {
        let mut manager = cert_manager.write().unwrap();
        manager.log("Successfully handled /api/cluster request");
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({ "data": info })),
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
                manager.log("Available endpoints:");
                manager.log("  - /health");
                manager.log("  - /api/cluster");
                manager.log("  - /swagger-ui");
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

