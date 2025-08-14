use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Path, Query},
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::api::server::{AppState, ApiServer};
use crate::api::handlers::pods::ListOptions;
use crate::scale::scale::Service;

pub async fn list_services(
    Extension(state): Extension<Arc<AppState>>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    match state.service_discovery.list_services().await {
        Ok(services) => {
            let response = ApiServer::list_response("Service", services, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to list services: {}", e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn list_namespaced_services(
    Extension(state): Extension<Arc<AppState>>,
    Path(namespace): Path<String>,
    Query(options): Query<ListOptions>,
) -> impl IntoResponse {
    match state.service_discovery.list_services_in_namespace(&namespace).await {
        Ok(services) => {
            let response = ApiServer::list_response("Service", services, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to list services in namespace {}: {}", namespace, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_service(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.service_discovery.get_service(&name).await {
        Ok(Some(service)) => {
            let response = ApiServer::success_response("Service", service);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Service '{}' not found", name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get service {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn get_namespaced_service(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.service_discovery.get_service_in_namespace(&namespace, &name).await {
        Ok(Some(service)) => {
            let response = ApiServer::success_response("Service", service);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Service '{}/{}' not found", namespace, name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to get service {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn create_service(
    Extension(state): Extension<Arc<AppState>>,
    Json(service): Json<Service>,
) -> impl IntoResponse {
    match state.service_discovery.create_service(service).await {
        Ok(created_service) => {
            let response = ApiServer::success_response("Service", created_service);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::BAD_REQUEST,
                "BadRequest",
                &format!("Failed to create service: {}", e),
            );
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn create_namespaced_service(
    Extension(state): Extension<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(mut service): Json<Service>,
) -> impl IntoResponse {
    // Set the namespace from the path
    service.metadata.namespace = Some(namespace.clone());
    
    match state.service_discovery.create_service(service).await {
        Ok(created_service) => {
            let response = ApiServer::success_response("Service", created_service);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::BAD_REQUEST,
                "BadRequest",
                &format!("Failed to create service in namespace {}: {}", namespace, e),
            );
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn update_service(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
    Json(service): Json<Service>,
) -> impl IntoResponse {
    if service.metadata.name != name {
        let error = ApiServer::error_response(
            StatusCode::BAD_REQUEST,
            "BadRequest",
            "Service name in body does not match name in URL",
        );
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    
    match state.service_discovery.update_service(service).await {
        Ok(updated_service) => {
            let response = ApiServer::success_response("Service", updated_service);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to update service {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn update_namespaced_service(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut service): Json<Service>,
) -> impl IntoResponse {
    if service.metadata.name != name {
        let error = ApiServer::error_response(
            StatusCode::BAD_REQUEST,
            "BadRequest",
            "Service name in body does not match name in URL",
        );
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    
    if service.metadata.namespace.as_ref() != Some(&namespace) {
        service.metadata.namespace = Some(namespace.clone());
    }
    
    match state.service_discovery.update_service(service).await {
        Ok(updated_service) => {
            let response = ApiServer::success_response("Service", updated_service);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to update service {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_service(
    Extension(state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.service_discovery.delete_service(&name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({
                "status": "Success",
                "message": format!("Service '{}' deleted", name)
            }));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Service '{}' not found", name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to delete service {}: {}", name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_namespaced_service(
    Extension(state): Extension<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.service_discovery.delete_service_in_namespace(&namespace, &name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({
                "status": "Success",
                "message": format!("Service '{}/{}' deleted", namespace, name)
            }));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(
                StatusCode::NOT_FOUND,
                "NotFound",
                &format!("Service '{}/{}' not found", namespace, name),
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &format!("Failed to delete service {}/{}: {}", namespace, name, e),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}