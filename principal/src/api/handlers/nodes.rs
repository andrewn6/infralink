use axum::{Json, http::StatusCode, response::IntoResponse, extract::{Extension, Path, Query}};
use std::sync::Arc;
use crate::api::server::{AppState, ApiServer};
use crate::api::handlers::pods::ListOptions;

// Node handlers - placeholder implementations
pub async fn list_nodes(Extension(state): Extension<Arc<AppState>>, Query(options): Query<ListOptions>) -> impl IntoResponse {
    let error = ApiServer::error_response(StatusCode::NOT_IMPLEMENTED, "NotImplemented", "Node operations not yet implemented");
    (StatusCode::NOT_IMPLEMENTED, Json(error))
}

pub async fn get_node(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    let error = ApiServer::error_response(StatusCode::NOT_IMPLEMENTED, "NotImplemented", "Node operations not yet implemented");
    (StatusCode::NOT_IMPLEMENTED, Json(error))
}

pub async fn patch_node(Extension(state): Extension<Arc<AppState>>, Path(name): Path<String>, Json(patch): Json<serde_json::Value>) -> impl IntoResponse {
    let error = ApiServer::error_response(StatusCode::NOT_IMPLEMENTED, "NotImplemented", "Node operations not yet implemented");
    (StatusCode::NOT_IMPLEMENTED, Json(error))
}