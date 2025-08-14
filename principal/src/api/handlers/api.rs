use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    extract::Extension,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::api::server::{AppState, ApiServer};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiVersions {
    pub kind: String,
    pub api_version: String,
    pub versions: Vec<String>,
    pub server_address_by_client_cidrs: Vec<ServerAddressByClientCidr>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerAddressByClientCidr {
    pub client_cidr: String,
    pub server_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResourceList {
    pub kind: String,
    pub api_version: String,
    pub group_version: String,
    pub resources: Vec<ApiResource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResource {
    pub name: String,
    pub singular_name: String,
    pub namespaced: bool,
    pub kind: String,
    pub verbs: Vec<String>,
    pub short_names: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub major: String,
    pub minor: String,
    pub git_version: String,
    pub git_commit: String,
    pub git_tree_state: String,
    pub build_date: String,
    pub go_version: String,
    pub compiler: String,
    pub platform: String,
}

pub async fn get_root() -> impl IntoResponse {
    Json(serde_json::json!({
        "paths": [
            "/api",
            "/api/v1",
            "/apis",
            "/healthz",
            "/livez",
            "/readyz",
            "/version"
        ]
    }))
}

pub async fn get_api_versions() -> impl IntoResponse {
    let versions = ApiVersions {
        kind: "APIVersions".to_string(),
        api_version: "v1".to_string(),
        versions: vec!["v1".to_string()],
        server_address_by_client_cidrs: vec![
            ServerAddressByClientCidr {
                client_cidr: "0.0.0.0/0".to_string(),
                server_address: "localhost:8080".to_string(),
            }
        ],
    };
    
    Json(versions)
}

pub async fn get_api_resources() -> impl IntoResponse {
    let resources = ApiResourceList {
        kind: "APIResourceList".to_string(),
        api_version: "v1".to_string(),
        group_version: "v1".to_string(),
        resources: vec![
            ApiResource {
                name: "pods".to_string(),
                singular_name: "pod".to_string(),
                namespaced: true,
                kind: "Pod".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["po".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "services".to_string(),
                singular_name: "service".to_string(),
                namespaced: true,
                kind: "Service".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["svc".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "deployments".to_string(),
                singular_name: "deployment".to_string(),
                namespaced: true,
                kind: "Deployment".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["deploy".to_string()]),
                categories: Some(vec!["all".to_string()]),
            },
            ApiResource {
                name: "configmaps".to_string(),
                singular_name: "configmap".to_string(),
                namespaced: true,
                kind: "ConfigMap".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["cm".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "secrets".to_string(),
                singular_name: "secret".to_string(),
                namespaced: true,
                kind: "Secret".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: None,
                categories: None,
            },
            ApiResource {
                name: "persistentvolumes".to_string(),
                singular_name: "persistentvolume".to_string(),
                namespaced: false,
                kind: "PersistentVolume".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["pv".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "persistentvolumeclaims".to_string(),
                singular_name: "persistentvolumeclaim".to_string(),
                namespaced: true,
                kind: "PersistentVolumeClaim".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["pvc".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "nodes".to_string(),
                singular_name: "node".to_string(),
                namespaced: false,
                kind: "Node".to_string(),
                verbs: vec!["get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["no".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "namespaces".to_string(),
                singular_name: "namespace".to_string(),
                namespaced: false,
                kind: "Namespace".to_string(),
                verbs: vec!["create", "delete", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["ns".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "events".to_string(),
                singular_name: "event".to_string(),
                namespaced: true,
                kind: "Event".to_string(),
                verbs: vec!["get", "list", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["ev".to_string()]),
                categories: None,
            },
            ApiResource {
                name: "horizontalpodautoscalers".to_string(),
                singular_name: "horizontalpodautoscaler".to_string(),
                namespaced: true,
                kind: "HorizontalPodAutoscaler".to_string(),
                verbs: vec!["create", "delete", "deletecollection", "get", "list", "patch", "update", "watch"].iter().map(|s| s.to_string()).collect(),
                short_names: Some(vec!["hpa".to_string()]),
                categories: None,
            },
        ],
    };
    
    Json(resources)
}

pub async fn get_version() -> impl IntoResponse {
    let version = Version {
        major: "1".to_string(),
        minor: "0".to_string(),
        git_version: "v1.0.0".to_string(),
        git_commit: "abc123".to_string(),
        git_tree_state: "clean".to_string(),
        build_date: chrono::Utc::now().to_rfc3339(),
        go_version: "go1.20.0".to_string(),
        compiler: "rustc".to_string(),
        platform: "linux/amd64".to_string(),
    };
    
    Json(version)
}