use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    extract::Extension,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::api::server::{AppState, ApiServer};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
    pub duration_ms: Option<u64>,
}

pub async fn health_check(
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let mut checks = Vec::new();
    let mut overall_healthy = true;
    
    // Check scheduler health
    let scheduler_start = std::time::Instant::now();
    let scheduler_healthy = state.scheduler.health_check().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "scheduler".to_string(),
        status: if scheduler_healthy { "healthy" } else { "unhealthy" }.to_string(),
        message: if scheduler_healthy { None } else { Some("Scheduler not responding".to_string()) },
        duration_ms: Some(scheduler_start.elapsed().as_millis() as u64),
    });
    overall_healthy &= scheduler_healthy;
    
    // Check autoscaler health
    let autoscaler_start = std::time::Instant::now();
    let autoscaler_healthy = state.autoscaler.health_check().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "autoscaler".to_string(),
        status: if autoscaler_healthy { "healthy" } else { "unhealthy" }.to_string(),
        message: if autoscaler_healthy { None } else { Some("Autoscaler not responding".to_string()) },
        duration_ms: Some(autoscaler_start.elapsed().as_millis() as u64),
    });
    overall_healthy &= autoscaler_healthy;
    
    // Check volume manager health
    let volume_start = std::time::Instant::now();
    let volume_healthy = state.volume_manager.health_check().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "volume_manager".to_string(),
        status: if volume_healthy { "healthy" } else { "unhealthy" }.to_string(),
        message: if volume_healthy { None } else { Some("Volume manager not responding".to_string()) },
        duration_ms: Some(volume_start.elapsed().as_millis() as u64),
    });
    overall_healthy &= volume_healthy;
    
    // Check ingress controller health
    let ingress_start = std::time::Instant::now();
    let ingress_healthy = state.ingress_controller.health_check().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "ingress_controller".to_string(),
        status: if ingress_healthy { "healthy" } else { "unhealthy" }.to_string(),
        message: if ingress_healthy { None } else { Some("Ingress controller not responding".to_string()) },
        duration_ms: Some(ingress_start.elapsed().as_millis() as u64),
    });
    overall_healthy &= ingress_healthy;
    
    // Check service discovery health
    let discovery_start = std::time::Instant::now();
    let discovery_healthy = state.service_discovery.health_check().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "service_discovery".to_string(),
        status: if discovery_healthy { "healthy" } else { "unhealthy" }.to_string(),
        message: if discovery_healthy { None } else { Some("Service discovery not responding".to_string()) },
        duration_ms: Some(discovery_start.elapsed().as_millis() as u64),
    });
    overall_healthy &= discovery_healthy;
    
    // Check metrics collector health
    let metrics_start = std::time::Instant::now();
    let metrics_healthy = state.metrics_collector.health_check().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "metrics_collector".to_string(),
        status: if metrics_healthy { "healthy" } else { "unhealthy" }.to_string(),
        message: if metrics_healthy { None } else { Some("Metrics collector not responding".to_string()) },
        duration_ms: Some(metrics_start.elapsed().as_millis() as u64),
    });
    overall_healthy &= metrics_healthy;
    
    let health_status = HealthStatus {
        status: if overall_healthy { "healthy" } else { "unhealthy" }.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        checks,
    };
    
    let status_code = if overall_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(health_status))
}

pub async fn readiness_check(
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    // Check if all critical components are ready
    let scheduler_ready = state.scheduler.is_ready().await.unwrap_or(false);
    let autoscaler_ready = state.autoscaler.is_ready().await.unwrap_or(false);
    let volume_ready = state.volume_manager.is_ready().await.unwrap_or(false);
    
    let all_ready = scheduler_ready && autoscaler_ready && volume_ready;
    
    let status = serde_json::json!({
        "status": if all_ready { "ready" } else { "not ready" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "scheduler": if scheduler_ready { "ready" } else { "not ready" },
            "autoscaler": if autoscaler_ready { "ready" } else { "not ready" },
            "volume_manager": if volume_ready { "ready" } else { "not ready" }
        }
    });
    
    let status_code = if all_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(status))
}

pub async fn liveness_check() -> impl IntoResponse {
    // Simple liveness check - if we can respond, we're alive
    let status = serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Json(status)
}