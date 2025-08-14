use std::sync::Arc;
use axum::{
    http::{StatusCode, Method},
    routing::{get, post, put, delete, patch},
    Router, Server,
    extract::Extension,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tokio::net::TcpListener;
use serde::{Serialize, Deserialize};

use crate::api::{handlers, middleware, routes};
use crate::services::{
    autoscaler::AutoscalerManager,
    storage::PersistentVolumeManager,
    ingress::IngressController,
    discovery::ServiceDiscovery,
    metrics::MetricsCollector,
};
use crate::scale::scale::Scheduler;

/// API Server providing Kubernetes-compatible REST endpoints
#[derive(Clone)]
pub struct ApiServer {
    pub config: ApiServerConfig,
    pub app_state: Arc<AppState>,
}

#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub enable_compression: bool,
    pub enable_tracing: bool,
    pub api_version: String,
    pub max_request_size: usize,
    pub request_timeout: std::time::Duration,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            enable_cors: true,
            enable_compression: true,
            enable_tracing: true,
            api_version: "v1".to_string(),
            max_request_size: 10 * 1024 * 1024, // 10MB
            request_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Shared application state accessible to all handlers
#[derive(Clone)]
pub struct AppState {
    pub scheduler: Arc<Scheduler>,
    pub autoscaler: Arc<AutoscalerManager>,
    pub volume_manager: Arc<PersistentVolumeManager>,
    pub ingress_controller: Arc<IngressController>,
    pub service_discovery: Arc<ServiceDiscovery>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub config: ApiServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub api_version: String,
    pub kind: String,
    pub metadata: Option<ResponseMetadata>,
    pub items: Option<Vec<T>>,
    pub data: Option<T>,
    pub status: Option<ApiStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub resource_version: String,
    pub continue_token: Option<String>,
    pub remaining_item_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStatus {
    pub kind: String,
    pub api_version: String,
    pub metadata: StatusMetadata,
    pub status: String,
    pub message: Option<String>,
    pub reason: Option<String>,
    pub code: Option<u16>,
    pub details: Option<StatusDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusMetadata {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub uid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusDetails {
    pub name: Option<String>,
    pub group: Option<String>,
    pub kind: Option<String>,
    pub uid: Option<String>,
    pub causes: Option<Vec<StatusCause>>,
    pub retry_after_seconds: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCause {
    pub reason: String,
    pub message: String,
    pub field: Option<String>,
}

impl ApiServer {
    pub fn new(config: ApiServerConfig, app_state: AppState) -> Self {
        Self {
            config,
            app_state: Arc::new(app_state),
        }
    }

    /// Start the API server
    pub async fn start(&self) -> Result<(), ApiServerError> {
        let app = self.create_router().await;
        let addr = format!("{}:{}", self.config.host, self.config.port);
        
        println!("🚀 Infralink API Server starting on {}", addr);
        println!("📚 API documentation available at http://{}/api/v1/docs", addr);
        println!("🔍 Health check endpoint: http://{}/api/v1/healthz", addr);
        
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| ApiServerError::StartupFailed(format!("Failed to bind to {}: {}", addr, e)))?;

        axum::serve(listener, app).await
            .map_err(|e| ApiServerError::StartupFailed(format!("Server failed: {}", e)))?;

        Ok(())
    }

    /// Create the main router with all routes and middleware
    async fn create_router(&self) -> Router {
        let api_v1_routes = self.create_api_v1_routes().await;
        let core_routes = self.create_core_routes().await;

        let mut app = Router::new()
            .nest("/api/v1", api_v1_routes)
            .merge(core_routes)
            .layer(Extension(self.app_state.clone()));

        // Add middleware layers
        let service_builder = ServiceBuilder::new();

        if self.config.enable_tracing {
            app = app.layer(
                service_builder.layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                uri = %request.uri(),
                                version = ?request.version(),
                            )
                        })
                )
            );
        }

        if self.config.enable_compression {
            app = app.layer(CompressionLayer::new());
        }

        if self.config.enable_cors {
            app = app.layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
                    .allow_headers(Any)
            );
        }

        // Add custom middleware
        app = app.layer(axum::middleware::from_fn(middleware::request_id));
        app = app.layer(axum::middleware::from_fn(middleware::request_logging));

        app
    }

    /// Create API v1 routes (Kubernetes-compatible)
    async fn create_api_v1_routes(&self) -> Router {
        Router::new()
            // Core API discovery
            .route("/", get(handlers::api::get_api_versions))
            .route("/api", get(handlers::api::get_api_resources))
            
            // Pod management
            .route("/pods", get(handlers::pods::list_pods))
            .route("/pods", post(handlers::pods::create_pod))
            .route("/pods/:name", get(handlers::pods::get_pod))
            .route("/pods/:name", put(handlers::pods::update_pod))
            .route("/pods/:name", delete(handlers::pods::delete_pod))
            .route("/pods/:name/status", patch(handlers::pods::patch_pod_status))
            .route("/pods/:name/log", get(handlers::pods::get_pod_logs))
            .route("/pods/:name/exec", post(handlers::pods::exec_pod))
            
            // Namespaced pod management
            .route("/namespaces/:namespace/pods", get(handlers::pods::list_namespaced_pods))
            .route("/namespaces/:namespace/pods", post(handlers::pods::create_namespaced_pod))
            .route("/namespaces/:namespace/pods/:name", get(handlers::pods::get_namespaced_pod))
            .route("/namespaces/:namespace/pods/:name", put(handlers::pods::update_namespaced_pod))
            .route("/namespaces/:namespace/pods/:name", delete(handlers::pods::delete_namespaced_pod))
            
            // Service management
            .route("/services", get(handlers::services::list_services))
            .route("/services", post(handlers::services::create_service))
            .route("/services/:name", get(handlers::services::get_service))
            .route("/services/:name", put(handlers::services::update_service))
            .route("/services/:name", delete(handlers::services::delete_service))
            
            // Namespaced service management
            .route("/namespaces/:namespace/services", get(handlers::services::list_namespaced_services))
            .route("/namespaces/:namespace/services", post(handlers::services::create_namespaced_service))
            .route("/namespaces/:namespace/services/:name", get(handlers::services::get_namespaced_service))
            .route("/namespaces/:namespace/services/:name", put(handlers::services::update_namespaced_service))
            .route("/namespaces/:namespace/services/:name", delete(handlers::services::delete_namespaced_service))
            
            // Deployment management  
            .route("/deployments", get(handlers::deployments::list_deployments))
            .route("/deployments", post(handlers::deployments::create_deployment))
            .route("/deployments/:name", get(handlers::deployments::get_deployment))
            .route("/deployments/:name", put(handlers::deployments::update_deployment))
            .route("/deployments/:name", delete(handlers::deployments::delete_deployment))
            .route("/deployments/:name/scale", patch(handlers::deployments::scale_deployment))
            
            // Namespaced deployment management
            .route("/namespaces/:namespace/deployments", get(handlers::deployments::list_namespaced_deployments))
            .route("/namespaces/:namespace/deployments", post(handlers::deployments::create_namespaced_deployment))
            .route("/namespaces/:namespace/deployments/:name", get(handlers::deployments::get_namespaced_deployment))
            .route("/namespaces/:namespace/deployments/:name", put(handlers::deployments::update_namespaced_deployment))
            .route("/namespaces/:namespace/deployments/:name", delete(handlers::deployments::delete_namespaced_deployment))
            
            // ConfigMap management
            .route("/configmaps", get(handlers::configmaps::list_configmaps))
            .route("/configmaps", post(handlers::configmaps::create_configmap))
            .route("/configmaps/:name", get(handlers::configmaps::get_configmap))
            .route("/configmaps/:name", put(handlers::configmaps::update_configmap))
            .route("/configmaps/:name", delete(handlers::configmaps::delete_configmap))
            
            // Secret management
            .route("/secrets", get(handlers::secrets::list_secrets))
            .route("/secrets", post(handlers::secrets::create_secret))
            .route("/secrets/:name", get(handlers::secrets::get_secret))
            .route("/secrets/:name", put(handlers::secrets::update_secret))
            .route("/secrets/:name", delete(handlers::secrets::delete_secret))
            
            // Persistent Volume management
            .route("/persistentvolumes", get(handlers::volumes::list_persistent_volumes))
            .route("/persistentvolumes", post(handlers::volumes::create_persistent_volume))
            .route("/persistentvolumes/:name", get(handlers::volumes::get_persistent_volume))
            .route("/persistentvolumes/:name", put(handlers::volumes::update_persistent_volume))
            .route("/persistentvolumes/:name", delete(handlers::volumes::delete_persistent_volume))
            
            // Persistent Volume Claim management
            .route("/persistentvolumeclaims", get(handlers::volumes::list_persistent_volume_claims))
            .route("/persistentvolumeclaims", post(handlers::volumes::create_persistent_volume_claim))
            .route("/persistentvolumeclaims/:name", get(handlers::volumes::get_persistent_volume_claim))
            .route("/persistentvolumeclaims/:name", put(handlers::volumes::update_persistent_volume_claim))
            .route("/persistentvolumeclaims/:name", delete(handlers::volumes::delete_persistent_volume_claim))
            
            // Ingress management
            .route("/ingresses", get(handlers::ingress::list_ingresses))
            .route("/ingresses", post(handlers::ingress::create_ingress))
            .route("/ingresses/:name", get(handlers::ingress::get_ingress))
            .route("/ingresses/:name", put(handlers::ingress::update_ingress))
            .route("/ingresses/:name", delete(handlers::ingress::delete_ingress))
            
            // HPA management
            .route("/horizontalpodautoscalers", get(handlers::autoscaling::list_hpas))
            .route("/horizontalpodautoscalers", post(handlers::autoscaling::create_hpa))
            .route("/horizontalpodautoscalers/:name", get(handlers::autoscaling::get_hpa))
            .route("/horizontalpodautoscalers/:name", put(handlers::autoscaling::update_hpa))
            .route("/horizontalpodautoscalers/:name", delete(handlers::autoscaling::delete_hpa))
            
            // Node management
            .route("/nodes", get(handlers::nodes::list_nodes))
            .route("/nodes/:name", get(handlers::nodes::get_node))
            .route("/nodes/:name", patch(handlers::nodes::patch_node))
            
            // Namespace management
            .route("/namespaces", get(handlers::namespaces::list_namespaces))
            .route("/namespaces", post(handlers::namespaces::create_namespace))
            .route("/namespaces/:name", get(handlers::namespaces::get_namespace))
            .route("/namespaces/:name", put(handlers::namespaces::update_namespace))
            .route("/namespaces/:name", delete(handlers::namespaces::delete_namespace))
            
            // Events
            .route("/events", get(handlers::events::list_events))
            .route("/namespaces/:namespace/events", get(handlers::events::list_namespaced_events))
            
            // Metrics
            .route("/metrics", get(handlers::metrics::get_metrics))
            .route("/metrics/nodes", get(handlers::metrics::get_node_metrics))
            .route("/metrics/pods", get(handlers::metrics::get_pod_metrics))
    }

    /// Create core routes (health, docs, etc.)
    async fn create_core_routes(&self) -> Router {
        Router::new()
            // Health and readiness checks
            .route("/healthz", get(handlers::health::health_check))
            .route("/readyz", get(handlers::health::readiness_check))
            .route("/livez", get(handlers::health::liveness_check))
            
            // API discovery and documentation
            .route("/", get(handlers::api::get_root))
            .route("/version", get(handlers::api::get_version))
            .route("/api/v1/docs", get(handlers::docs::get_api_docs))
            .route("/openapi.json", get(handlers::docs::get_openapi_spec))
            
            // WebSocket endpoints for streaming
            .route("/api/v1/watch/pods", get(handlers::watch::watch_pods))
            .route("/api/v1/watch/services", get(handlers::watch::watch_services))
            .route("/api/v1/watch/deployments", get(handlers::watch::watch_deployments))
            .route("/api/v1/watch/events", get(handlers::watch::watch_events))
    }

    /// Create a success response
    pub fn success_response<T: Serialize>(
        kind: &str,
        data: T,
    ) -> ApiResponse<T> {
        ApiResponse {
            api_version: "v1".to_string(),
            kind: kind.to_string(),
            metadata: None,
            items: None,
            data: Some(data),
            status: None,
        }
    }

    /// Create a list response
    pub fn list_response<T: Serialize>(
        kind: &str,
        items: Vec<T>,
        metadata: Option<ResponseMetadata>,
    ) -> ApiResponse<T> {
        ApiResponse {
            api_version: "v1".to_string(),
            kind: format!("{}List", kind),
            metadata,
            items: Some(items),
            data: None,
            status: None,
        }
    }

    /// Create an error response
    pub fn error_response(
        status_code: StatusCode,
        reason: &str,
        message: &str,
    ) -> ApiResponse<()> {
        ApiResponse {
            api_version: "v1".to_string(),
            kind: "Status".to_string(),
            metadata: None,
            items: None,
            data: None,
            status: Some(ApiStatus {
                kind: "Status".to_string(),
                api_version: "v1".to_string(),
                metadata: StatusMetadata {
                    name: None,
                    namespace: None,
                    uid: None,
                },
                status: "Failure".to_string(),
                message: Some(message.to_string()),
                reason: Some(reason.to_string()),
                code: Some(status_code.as_u16()),
                details: None,
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiServerError {
    #[error("API server startup failed: {0}")]
    StartupFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Internal server error: {0}")]
    InternalError(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}