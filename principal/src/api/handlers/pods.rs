use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Path, Query},
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::api::server::{AppState, ApiServer};
use crate::scale::scale::{Pod, PodSpec, PodStatus, Container, ContainerStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListOptions {
    pub limit: Option<u32>,
    pub continue_token: Option<String>,
    pub field_selector: Option<String>,
    pub label_selector: Option<String>,
    pub resource_version: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub watch: Option<bool>,
}

pub async fn list_pods(
    Extension(state): Extension<Arc<AppState>>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    match state.scheduler.list_pods().await {
        Ok(pods) => {
            let response = ApiServer::list_response("Pod", pods, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to list pods: {}", e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn list_namespaced_pods(
    Extension(state): Extension<Arc<AppState>>,
    Path(namespace): Path<String>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    match state.scheduler.list_pods_in_namespace(&namespace).await {
        Ok(pods) => {
            let response = ApiServer::list_response("Pod", pods, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to list pods in namespace {}: {}", namespace, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.scheduler.get_pod(&name).await {
        Ok(Some(pod)) => {
            let response = ApiServer::success_response("Pod", pod);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Pod '{}' not found", name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get pod {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_namespaced_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.scheduler.get_pod_in_namespace(&namespace, &name).await {
        Ok(Some(pod)) => {
            let response = ApiServer::success_response("Pod", pod);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Pod '{}/{}' not found", namespace, name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get pod {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn create_pod(
    Extension(state): Extension<Arc<AppState>>,
    Json(pod): Json<Pod>,
) -> impl IntoResponse {
    match state.scheduler.create_pod(pod).await {
        Ok(created_pod) => {
            let response = ApiServer::success_response("Pod", created_pod);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::BAD_REQUEST,
                "BadRequest",
                &format!("Failed to create pod: {}", e),
            );
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn create_namespaced_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(mut pod): Json<Pod>,
) -> impl IntoResponse {
    // Set the namespace from the path
    pod.metadata.namespace = Some(namespace.clone());
    
    match state.scheduler.create_pod(pod).await {
        Ok(created_pod) => {
            let response = ApiServer::success_response("Pod", created_pod);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::BAD_REQUEST,
                "BadRequest",
                &format!("Failed to create pod in namespace {}: {}", namespace, e),
            );
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn update_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
    Json(pod): Json<Pod>,
) -> impl IntoResponse {
    if pod.metadata.name != name {
        let error = ApiServer::error_response(
            StatusCode::BAD_REQUEST,
            "BadRequest",
            "Pod name in body does not match name in URL",
        );
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    
    match state.scheduler.update_pod(pod).await {
        Ok(updated_pod) => {
            let response = ApiServer::success_response("Pod", updated_pod);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to update pod {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn update_namespaced_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut pod): Json<Pod>,
) -> impl IntoResponse {
    if pod.metadata.name != name {
        let error = ApiServer::error_response(
            StatusCode::BAD_REQUEST,
            "BadRequest",
            "Pod name in body does not match name in URL",
        );
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    
    if pod.metadata.namespace.as_ref() != Some(&namespace) {
        pod.metadata.namespace = Some(namespace.clone());
    }
    
    match state.scheduler.update_pod(pod).await {
        Ok(updated_pod) => {
            let response = ApiServer::success_response("Pod", updated_pod);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to update pod {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.scheduler.delete_pod(&name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({
                "status": "Success",
                "message": format!("Pod '{}' deleted", name)
            }));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Pod '{}' not found", name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to delete pod {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_namespaced_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.scheduler.delete_pod_in_namespace(&namespace, &name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({
                "status": "Success",
                "message": format!("Pod '{}/{}' deleted", namespace, name)
            }));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Pod '{}/{}' not found", namespace, name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to delete pod {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn patch_pod_status(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
    Json(status): Json<PodStatus>,
) -> impl IntoResponse {
    match state.scheduler.update_pod_status(&name, status).await {
        Ok(updated_pod) => {
            let response = ApiServer::success_response("Pod", updated_pod);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to update pod status for {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_pod_logs(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let container = params.get("container");
    let follow = params.get("follow").map(|s| s == "true").unwrap_or(false);
    let tail_lines = params.get("tailLines").and_then(|s| s.parse().ok());
    let since_seconds = params.get("sinceSeconds").and_then(|s| s.parse().ok());
    
    match state.scheduler.get_pod_logs(&name, container, follow, tail_lines, since_seconds).await {
        Ok(logs) => {
            // Return logs as plain text
            axum::response::Response::builder()
                .header("content-type", "text/plain")
                .body(logs)
                .unwrap()
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get logs for pod {}: {}", name, e),
            );
            axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error).unwrap())
                .unwrap()
        }
    }
}

pub async fn exec_pod(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
    Json(exec_request): Json<PodExecRequest>,
) -> impl IntoResponse {
    match state.scheduler.exec_pod(&name, &exec_request.container.unwrap_or_default(), exec_request.command).await {
        Ok(output) => {
            let response = ApiServer::success_response("ExecOutput", serde_json::json!({
                "stdout": output.stdout,
                "stderr": output.stderr,
                "exit_code": output.exit_code
            }));
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to exec in pod {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodExecRequest {
    pub container: Option<String>,
    pub command: Vec<String>,
    pub stdin: Option<bool>,
    pub stdout: Option<bool>,
    pub stderr: Option<bool>,
    pub tty: Option<bool>,
}