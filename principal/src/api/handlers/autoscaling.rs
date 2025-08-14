use axum::{Json, http::StatusCode, response::IntoResponse, extract::{Extension, Path, Query}};
use std::sync::Arc;
use crate::api::server::{AppState, ApiServer};
use crate::api::handlers::pods::ListOptions;
use crate::services::autoscaler::HorizontalPodAutoscalerSpec;

pub async fn list_hpas(Extension(state): Extension<Arc<AppState>>, Query(options): Query<ListOptions>) -> impl IntoResponse {
    match state.autoscaler.hpa_controller.list_hpas().await {
        Ok(hpas) => {
            let response = ApiServer::list_response("HorizontalPodAutoscaler", hpas, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to list HPAs: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn create_hpa(Extension(state): Extension<Arc<AppState>>, Json(hpa): Json<HorizontalPodAutoscalerSpec>) -> impl IntoResponse {
    match state.autoscaler.hpa_controller.create_hpa(hpa).await {
        Ok(created_hpa) => {
            let response = ApiServer::success_response("HorizontalPodAutoscaler", created_hpa);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", &format!("Failed to create HPA: {}", e));
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn get_hpa(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.autoscaler.hpa_controller.get_hpa(&name).await {
        Ok(Some(hpa)) => {
            let response = ApiServer::success_response("HorizontalPodAutoscaler", hpa);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("HPA '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to get HPA {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn update_hpa(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>, Json(hpa): Json<HorizontalPodAutoscalerSpec>) -> impl IntoResponse {
    if hpa.name != name {
        let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", "HPA name in body does not match name in URL");
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    match state.autoscaler.hpa_controller.update_hpa(hpa).await {
        Ok(updated_hpa) => {
            let response = ApiServer::success_response("HorizontalPodAutoscaler", updated_hpa);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to update HPA {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_hpa(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.autoscaler.hpa_controller.delete_hpa(&name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({"status": "Success", "message": format!("HPA '{}' deleted", name)}));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("HPA '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to delete HPA {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}