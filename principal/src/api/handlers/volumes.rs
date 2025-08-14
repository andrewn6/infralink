use axum::{Json, http::StatusCode, response::IntoResponse, extract::{Extension, Path, Query}};
use std::sync::Arc;
use crate::api::server::{AppState, ApiServer};
use crate::api::handlers::pods::ListOptions;
use crate::services::storage::{PersistentVolume, PersistentVolumeClaim};

pub async fn list_persistent_volumes(Extension(state): Extension<Arc<AppState>>, Query(options): Query<ListOptions>) -> impl IntoResponse {
    match state.volume_manager.list_volumes().await {
        Ok(volumes) => {
            let response = ApiServer::list_response("PersistentVolume", volumes, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to list persistent volumes: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn create_persistent_volume(Extension(state): Extension<Arc<AppState>>, Json(volume): Json<PersistentVolume>) -> impl IntoResponse {
    match state.volume_manager.create_volume(volume).await {
        Ok(created_volume) => {
            let response = ApiServer::success_response("PersistentVolume", created_volume);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", &format!("Failed to create persistent volume: {}", e));
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn get_persistent_volume(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.volume_manager.get_volume(&name).await {
        Ok(Some(volume)) => {
            let response = ApiServer::success_response("PersistentVolume", volume);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("PersistentVolume '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to get persistent volume {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn update_persistent_volume(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>, Json(volume): Json<PersistentVolume>) -> impl IntoResponse {
    if volume.metadata.name != name {
        let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", "Volume name in body does not match name in URL");
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    match state.volume_manager.update_volume(volume).await {
        Ok(updated_volume) => {
            let response = ApiServer::success_response("PersistentVolume", updated_volume);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to update persistent volume {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_persistent_volume(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.volume_manager.delete_volume(&name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({"status": "Success", "message": format!("PersistentVolume '{}' deleted", name)}));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("PersistentVolume '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to delete persistent volume {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// PVC handlers
pub async fn list_persistent_volume_claims(Extension(state): Extension<Arc<AppState>>, Query(options): Query<ListOptions>) -> impl IntoResponse {
    match state.volume_manager.list_claims().await {
        Ok(claims) => {
            let response = ApiServer::list_response("PersistentVolumeClaim", claims, None);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to list persistent volume claims: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn create_persistent_volume_claim(Extension(state): Extension<Arc<AppState>>, Json(claim): Json<PersistentVolumeClaim>) -> impl IntoResponse {
    match state.volume_manager.create_claim(claim).await {
        Ok(created_claim) => {
            let response = ApiServer::success_response("PersistentVolumeClaim", created_claim);
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", &format!("Failed to create persistent volume claim: {}", e));
            (StatusCode::BAD_REQUEST, Json(error))
        }
    }
}

pub async fn get_persistent_volume_claim(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.volume_manager.get_claim(&name).await {
        Ok(Some(claim)) => {
            let response = ApiServer::success_response("PersistentVolumeClaim", claim);
            Json(response)
        }
        Ok(None) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("PersistentVolumeClaim '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to get persistent volume claim {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn update_persistent_volume_claim(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>, Json(claim): Json<PersistentVolumeClaim>) -> impl IntoResponse {
    if claim.metadata.name != name {
        let error = ApiServer::error_response(StatusCode::BAD_REQUEST, "BadRequest", "Claim name in body does not match name in URL");
        return (StatusCode::BAD_REQUEST, Json(error));
    }
    match state.volume_manager.update_claim(claim).await {
        Ok(updated_claim) => {
            let response = ApiServer::success_response("PersistentVolumeClaim", updated_claim);
            Json(response)
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to update persistent volume claim {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

pub async fn delete_persistent_volume_claim(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    match state.volume_manager.delete_claim(&name).await {
        Ok(true) => {
            let response = ApiServer::success_response("Status", serde_json::json!({"status": "Success", "message": format!("PersistentVolumeClaim '{}' deleted", name)}));
            Json(response)
        }
        Ok(false) => {
            let error = ApiServer::error_response(StatusCode::NOT_FOUND, "NotFound", &format!("PersistentVolumeClaim '{}' not found", name));
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiServer::error_response(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &format!("Failed to delete persistent volume claim {}: {}", name, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}