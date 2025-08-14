use principal::api::server::{ApiServer, ApiServerConfig, AppState};
use principal::services::{
    autoscaler::AutoscalerManager,
    storage::PersistentVolumeManager,
    ingress::IngressController,
    discovery::ServiceDiscovery,
    metrics::MetricsCollector,
};
use principal::scale::scale::Scheduler;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 Starting Infralink API Server Demo");
    
    // Create all the required services
    let scheduler = Arc::new(Scheduler::new("default".to_string()));
    let autoscaler = Arc::new(AutoscalerManager::new().await?);
    let volume_manager = Arc::new(PersistentVolumeManager::new().await?);
    let ingress_controller = Arc::new(IngressController::new().await?);
    let service_discovery = Arc::new(ServiceDiscovery::new().await?);
    let metrics_collector = Arc::new(MetricsCollector::new().await?);
    
    // Create API server configuration
    let config = ApiServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_cors: true,
        enable_compression: true,
        enable_tracing: true,
        api_version: "v1".to_string(),
        max_request_size: 10 * 1024 * 1024, // 10MB
        request_timeout: std::time::Duration::from_secs(30),
    };
    
    // Create application state
    let app_state = AppState {
        scheduler,
        autoscaler,
        volume_manager,
        ingress_controller,
        service_discovery,
        metrics_collector,
        config: config.clone(),
    };
    
    // Create and start the API server
    let api_server = ApiServer::new(config, app_state);
    
    println!("🎯 API Server configured successfully");
    println!("📚 Swagger UI will be available at: http://localhost:8080/api/v1/docs");
    println!("🔍 Health check at: http://localhost:8080/healthz");
    println!("📊 Metrics at: http://localhost:8080/api/v1/metrics");
    println!("🔌 WebSocket watch endpoints at: ws://localhost:8080/api/v1/watch/*");
    
    // Start the server (this will run forever)
    api_server.start().await?;
    
    Ok(())
}