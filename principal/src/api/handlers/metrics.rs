use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Query},
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::api::server::{AppState, ApiServer};
use crate::services::metrics::{PodMetrics, NodeMetrics};

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub field_selector: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub timestamp: String,
    pub nodes: Vec<NodeMetrics>,
    pub pods: Vec<PodMetrics>,
    pub cluster_totals: ClusterTotals,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterTotals {
    pub total_nodes: usize,
    pub total_pods: usize,
    pub total_cpu_capacity: f64,
    pub total_memory_capacity: u64,
    pub total_cpu_used: f64,
    pub total_memory_used: u64,
    pub cpu_utilization_percent: f64,
    pub memory_utilization_percent: f64,
}

pub async fn get_metrics(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<MetricsQuery>,
) -> impl IntoResponse {
    match collect_cluster_metrics(&state).await {
        Ok(metrics) => {
            let response = ApiServer::success_response("ClusterMetrics", metrics);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to collect cluster metrics: {}", e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_node_metrics(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<MetricsQuery>,
) -> impl IntoResponse {
    match state.metrics_collector.get_node_metrics().await {
        Ok(node_metrics) => {
            let metrics: Vec<NodeMetrics> = node_metrics.into_values().collect();
            let response = ApiServer::list_response("NodeMetrics", metrics, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get node metrics: {}", e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_pod_metrics(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<MetricsQuery>,
) -> impl IntoResponse {
    match state.metrics_collector.get_pod_metrics().await {
        Ok(pod_metrics) => {
            let mut metrics: Vec<PodMetrics> = pod_metrics.into_values().collect();
            
            // Filter by namespace if specified
            if let Some(namespace) = &query.namespace {
                metrics.retain(|m| m.namespace.as_ref() == Some(namespace));
            }
            
            let response = ApiServer::list_response("PodMetrics", metrics, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get pod metrics: {}", e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

async fn collect_cluster_metrics(state: &Arc<AppState>) -> Result<ClusterMetrics, Box<dyn std::error::Error + Send + Sync>> {
    let node_metrics = state.metrics_collector.get_node_metrics().await?;
    let pod_metrics = state.metrics_collector.get_pod_metrics().await?;
    
    let nodes: Vec<NodeMetrics> = node_metrics.values().cloned().collect();
    let pods: Vec<PodMetrics> = pod_metrics.values().cloned().collect();
    
    // Calculate cluster totals
    let total_nodes = nodes.len();
    let total_pods = pods.len();
    
    let total_cpu_capacity: f64 = nodes.iter().map(|n| n.cpu_capacity).sum();
    let total_memory_capacity: u64 = nodes.iter().map(|n| n.memory_capacity).sum();
    
    let total_cpu_used: f64 = nodes.iter().map(|n| n.cpu_usage).sum();
    let total_memory_used: u64 = nodes.iter().map(|n| n.memory_usage).sum();
    
    let cpu_utilization_percent = if total_cpu_capacity > 0.0 {
        (total_cpu_used / total_cpu_capacity) * 100.0
    } else {
        0.0
    };
    
    let memory_utilization_percent = if total_memory_capacity > 0 {
        (total_memory_used as f64 / total_memory_capacity as f64) * 100.0
    } else {
        0.0
    };
    
    let cluster_totals = ClusterTotals {
        total_nodes,
        total_pods,
        total_cpu_capacity,
        total_memory_capacity,
        total_cpu_used,
        total_memory_used,
        cpu_utilization_percent,
        memory_utilization_percent,
    };
    
    Ok(ClusterMetrics {
        timestamp: chrono::Utc::now().to_rfc3339(),
        nodes,
        pods,
        cluster_totals,
    })
}