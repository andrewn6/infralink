use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Path, Query},
};
use std::sync::Arc;

use crate::api::server::{AppState, ApiServer};
use crate::api::handlers::pods::ListOptions;
use crate::scale::scale::Deployment;

pub async fn list_deployments(
    Extension(state): Extension<Arc<AppState>>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    match state.scheduler.list_deployments().await {
        Ok(deployments) => {
            let response = ApiServer::list_response("Deployment", deployments, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to list deployments: {}", e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn list_namespaced_deployments(
    Extension(state): Extension<Arc<AppState>>,
    Path(namespace): Path<String>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    match state.scheduler.list_deployments_in_namespace(&namespace).await {
        Ok(deployments) => {
            let response = ApiServer::list_response("Deployment", deployments, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to list deployments in namespace {}: {}", namespace, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.scheduler.get_deployment(&name).await {
        Ok(Some(deployment)) => {
            let response = ApiServer::success_response("Deployment", deployment);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Deployment '{}' not found", name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get deployment {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_namespaced_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.scheduler.get_deployment_in_namespace(&namespace, &name).await {
        Ok(Some(deployment)) => {
            let response = ApiServer::success_response("Deployment", deployment);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Deployment '{}/{}' not found", namespace, name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get deployment {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn create_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Json(deployment): Json<Deployment>,
) -> impl IntoResponse {
    match state.scheduler.create_deployment(deployment).await {
        Ok(created_deployment) => {
            let response = ApiServer::success_response("Deployment", created_deployment);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::BAD_REQUEST,
                "BadRequest",
                &format!("Failed to create deployment: {}", e),
            );
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn create_namespaced_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(mut deployment): Json<Deployment>,
) -> impl IntoResponse {
    deployment.metadata.namespace = Some(namespace.clone());
    
    match state.scheduler.create_deployment(deployment).await {
        Ok(created_deployment) => {
            let response = ApiServer::success_response("Deployment", created_deployment);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::BAD_REQUEST,
                "BadRequest",
                &format!("Failed to create deployment in namespace {}: {}", namespace, e),
            );
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn update_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
    Json(deployment): Json<Deployment>,
) -> impl IntoResponse {
    if deployment.metadata.name != name {
        let error = ApiServer::error_response(
            StatusCode::BAD_REQUEST,
            "BadRequest",
            "Deployment name in body does not match name in URL",
        );
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    
    match state.scheduler.update_deployment(deployment).await {
        Ok(updated_deployment) => {
            let response = ApiServer::success_response("Deployment", updated_deployment);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to update deployment {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn update_namespaced_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut deployment): Json<Deployment>,
) -> impl IntoResponse {
    if deployment.metadata.name != name {
        let error = ApiServer::error_response(
            StatusCode::BAD_REQUEST,
            "BadRequest",
            "Deployment name in body does not match name in URL",
        );
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    
    if deployment.metadata.namespace.as_ref() != Some(&namespace) {
        deployment.metadata.namespace = Some(namespace.clone());
    }
    
    match state.scheduler.update_deployment(deployment).await {
        Ok(updated_deployment) => {
            let response = ApiServer::success_response("Deployment", updated_deployment);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to update deployment {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.scheduler.delete_deployment(&name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({
                "status": "Success",
                "message": format!("Deployment '{}' deleted", name)
            }));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Deployment '{}' not found", name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to delete deployment {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_namespaced_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.scheduler.delete_deployment_in_namespace(&namespace, &name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({
                "status": "Success",
                "message": format!("Deployment '{}/{}' deleted", namespace, name)
            }));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Deployment '{}/{}' not found", namespace, name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to delete deployment {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn scale_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
    Json(scale_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let replicas = scale_request.get("spec")
        .and_then(|spec| spec.get("replicas"))
        .and_then(|r| r.as_u64())
        .unwrap_or(1) as u32;
    
    match state.scheduler.scale_deployment(&name, replicas).await {
        Ok(updated_deployment) => {
            let response = ApiServer::success_response("Deployment", updated_deployment);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to scale deployment {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}