use axum::{
    extract::{
        Extension,
        Query,
        ws::{WebSocket, WebSocketUpgrade, Message},
    },
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use futures_util::{sink::SinkExt, stream::StreamExt};

use crate::api::server::{AppState, ApiServer};
use crate::api::handlers::pods::ListOptions;

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchEvent<T> {
    #[serde(rename = "type")]
    pub event_type: String,
    pub object: T,
}

pub async fn watch_pods(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_pod_watch(socket, state, options))
}

pub async fn watch_services(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_service_watch(socket, state, options))
}

pub async fn watch_deployments(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_deployment_watch(socket, state, options))
}

pub async fn watch_events(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_event_watch(socket, state, options))
}

async fn handle_pod_watch(
    mut socket: WebSocket,
    state: Arc<AppState>,
    _options: ListOptions,
) {
    let mut interval = interval(Duration::from_secs(5));
    let mut last_version = String::new();
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match state.scheduler.list_pods().await {
                    Ok(pods) => {
                        let current_version = format!("{}", chrono::Utc::now().timestamp());
                        
                        if current_version != last_version {
                            for pod in pods {
                                let event = WatchEvent {
                                    event_type: "MODIFIED".to_string(),
                                    object: pod,
                                };
                                
                                if let Ok(msg) = serde_json::to_string(&event) {
                                    if socket.send(Message::Text(msg)).await.is_err() {
                                        return; // Client disconnected
                                    }
                                }
                            }
                            last_version = current_version;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to list pods for watch: {}", e);
                        let error_msg = serde_json::json!({
                            "type": "ERROR",
                            "object": {
                                "kind": "Status",
                                "apiVersion": "v1",
                                "status": "Failure",
                                "message": format!("Failed to watch pods: {}", e)
                            }
                        });
                        
                        if let Ok(msg) = serde_json::to_string(&error_msg) {
                            let _ = socket.send(Message::Text(msg)).await;
                        }
                        return;
                    }
                }
            }
            
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        break; // Client closed connection
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {} // Ignore other message types
                }
            }
        }
    }
}

async fn handle_service_watch(
    mut socket: WebSocket,
    state: Arc<AppState>,
    _options: ListOptions,
) {
    let mut interval = interval(Duration::from_secs(5));
    let mut last_version = String::new();
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match state.service_discovery.list_services().await {
                    Ok(services) => {
                        let current_version = format!("{}", chrono::Utc::now().timestamp());
                        
                        if current_version != last_version {
                            for service in services {
                                let event = WatchEvent {
                                    event_type: "MODIFIED".to_string(),
                                    object: service,
                                };
                                
                                if let Ok(msg) = serde_json::to_string(&event) {
                                    if socket.send(Message::Text(msg)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                            last_version = current_version;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to list services for watch: {}", e);
                        return;
                    }
                }
            }
            
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_deployment_watch(
    mut socket: WebSocket,
    state: Arc<AppState>,
    _options: ListOptions,
) {
    let mut interval = interval(Duration::from_secs(5));
    let mut last_version = String::new();
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match state.scheduler.list_deployments().await {
                    Ok(deployments) => {
                        let current_version = format!("{}", chrono::Utc::now().timestamp());
                        
                        if current_version != last_version {
                            for deployment in deployments {
                                let event = WatchEvent {
                                    event_type: "MODIFIED".to_string(),
                                    object: deployment,
                                };
                                
                                if let Ok(msg) = serde_json::to_string(&event) {
                                    if socket.send(Message::Text(msg)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                            last_version = current_version;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to list deployments for watch: {}", e);
                        return;
                    }
                }
            }
            
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_event_watch(
    mut socket: WebSocket,
    state: Arc<AppState>,
    _options: ListOptions,
) {
    let mut interval = interval(Duration::from_secs(2));
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Send a heartbeat event to keep connection alive
                let heartbeat = serde_json::json!({
                    "type": "HEARTBEAT",
                    "object": {
                        "kind": "Event",
                        "apiVersion": "v1",
                        "metadata": {
                            "name": "heartbeat",
                            "namespace": "default"
                        },
                        "type": "Normal",
                        "reason": "Heartbeat",
                        "message": "WebSocket connection alive",
                        "firstTimestamp": chrono::Utc::now().to_rfc3339(),
                        "lastTimestamp": chrono::Utc::now().to_rfc3339(),
                        "count": 1
                    }
                });
                
                if let Ok(msg) = serde_json::to_string(&heartbeat) {
                    if socket.send(Message::Text(msg)).await.is_err() {
                        return;
                    }
                }
            }
            
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}