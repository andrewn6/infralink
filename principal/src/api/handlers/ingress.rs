use axum::{Json, http::StatusCode, response::IntoResponse, extract::{Extension, Path, Query}};
use std::sync::Arc;
use crate::api::server::{AppState, ApiServer};
use crate::api::handlers::pods::ListOptions;
use crate::services::ingress::IngressRule;

pub async fn list_ingresses(Extension(state): Extension<Arc<AppState>>, Query(options): Query<ListOptions>) -> impl IntoResponse {
    match state.ingress_controller.list_ingresses().await {
        Ok(ingresses) => {
            let response = ApiServer::list_response("Ingress", ingresses, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to list ingresses: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn create_ingress(Extension(state): Extension<Arc<AppState>>, Json(ingress): Json<IngressRule>) -> impl IntoResponse {
    match state.ingress_controller.create_ingress(ingress).await {
        Ok(created_ingress) => {
            let response = ApiServer::success_response("Ingress", created_ingress);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", &format!("Failed to create ingress: {}", e));
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn get_ingress(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.ingress_controller.get_ingress(&name).await {
        Ok(Some(ingress)) => {
            let response = ApiServer::success_response("Ingress", ingress);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("Ingress '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to get ingress {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn update_ingress(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>, Json(ingress): Json<IngressRule>) -> impl IntoResponse {
    if ingress.name != name {
        let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", "Ingress name in body does not match name in URL");
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    match state.ingress_controller.update_ingress(ingress).await {
        Ok(updated_ingress) => {
            let response = ApiServer::success_response("Ingress", updated_ingress);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to update ingress {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_ingress(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.ingress_controller.delete_ingress(&name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({"status": "Success", "message": format!("Ingress '{}' deleted", name)}));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("Ingress '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to delete ingress {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}