use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Html},
    extract::Extension,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::api::server::{AppState, ApiServer};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: ApiInfo,
    pub servers: Vec<ApiServer>,
    pub paths: serde_json::Value,
    pub components: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiInfo {
    pub title: String,
    pub description: String,
    pub version: String,
    pub contact: ApiContact,
    pub license: ApiLicense,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiContact {
    pub name: String,
    pub url: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiLicense {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiServerInfo {
    pub url: String,
    pub description: String,
}

pub async fn get_api_docs() -> impl IntoResponse {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Infralink API Documentation</title>
        <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui.css" />
        <style>
            html {
                box-sizing: border-box;
                overflow: -moz-scrollbars-vertical;
                overflow-y: scroll;
            }
            *, *:before, *:after {
                box-sizing: inherit;
            }
            body {
                margin:0;
                background: #fafafa;
            }
        </style>
    </head>
    <body>
        <div id="swagger-ui"></div>
        <script src="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui-bundle.js"></script>
        <script src="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui-standalone-preset.js"></script>
        <script>
            window.onload = function() {
                const ui = SwaggerUIBundle({
                    url: '/openapi.json',
                    dom_id: '#swagger-ui',
                    deepLinking: true,
                    presets: [
                        SwaggerUIBundle.presets.apis,
                        SwaggerUIStandalonePreset
                    ],
                    plugins: [
                        SwaggerUIBundle.plugins.DownloadUrl
                    ],
                    layout: "StandaloneLayout"
                });
            };
        </script>
    </body>
    </html>
    "#;
    
    Html(html)
}

pub async fn get_openapi_spec() -> impl IntoResponse {
    let spec = OpenApiSpec {
        openapi: "3.0.3".to_string(),
        info: ApiInfo {
            title: "Infralink API".to_string(),
            description: "Container orchestration platform with Kubernetes-compatible REST API".to_string(),
            version: "1.0.0".to_string(),
            contact: ApiContact {
                name: "Infralink Team".to_string(),
                url: "https://github.com/infralink/infralink".to_string(),
                email: "support@infralink.dev".to_string(),
            },
            license: ApiLicense {
                name: "MIT".to_string(),
                url: "https://opensource.org/licenses/MIT".to_string(),
            },
        },
        servers: vec![
            ApiServerInfo {
                url: "http://localhost:8080".to_string(),
                description: "Development server".to_string(),
            },
        ],
        paths: serde_json::json!({
            "/api/v1/pods": {
                "get": {
                    "summary": "List pods",
                    "description": "List all pods in the cluster",
                    "parameters": [
                        {
                            "name": "namespace",
                            "in": "query",
                            "description": "Filter by namespace",
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "labelSelector",
                            "in": "query",
                            "description": "Label selector for filtering",
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of pods",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/PodList"
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create pod",
                    "description": "Create a new pod",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/Pod"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Pod created",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/Pod"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/v1/services": {
                "get": {
                    "summary": "List services",
                    "description": "List all services in the cluster",
                    "responses": {
                        "200": {
                            "description": "List of services",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ServiceList"
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create service",
                    "description": "Create a new service",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/Service"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Service created"
                        }
                    }
                }
            },
            "/healthz": {
                "get": {
                    "summary": "Health check",
                    "description": "Check the health of the API server",
                    "responses": {
                        "200": {
                            "description": "API server is healthy"
                        },
                        "503": {
                            "description": "API server is unhealthy"
                        }
                    }
                }
            }
        }),
        components: serde_json::json!({
            "schemas": {
                "Pod": {
                    "type": "object",
                    "properties": {
                        "apiVersion": { "type": "string", "example": "v1" },
                        "kind": { "type": "string", "example": "Pod" },
                        "metadata": { "$ref": "#/components/schemas/ObjectMeta" },
                        "spec": { "$ref": "#/components/schemas/PodSpec" },
                        "status": { "$ref": "#/components/schemas/PodStatus" }
                    }
                },
                "PodList": {
                    "type": "object",
                    "properties": {
                        "apiVersion": { "type": "string", "example": "v1" },
                        "kind": { "type": "string", "example": "PodList" },
                        "items": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/Pod" }
                        }
                    }
                },
                "Service": {
                    "type": "object",
                    "properties": {
                        "apiVersion": { "type": "string", "example": "v1" },
                        "kind": { "type": "string", "example": "Service" },
                        "metadata": { "$ref": "#/components/schemas/ObjectMeta" },
                        "spec": { "$ref": "#/components/schemas/ServiceSpec" },
                        "status": { "$ref": "#/components/schemas/ServiceStatus" }
                    }
                },
                "ServiceList": {
                    "type": "object",
                    "properties": {
                        "apiVersion": { "type": "string", "example": "v1" },
                        "kind": { "type": "string", "example": "ServiceList" },
                        "items": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/Service" }
                        }
                    }
                },
                "ObjectMeta": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "namespace": { "type": "string" },
                        "labels": {
                            "type": "object",
                            "additionalProperties": { "type": "string" }
                        },
                        "annotations": {
                            "type": "object",
                            "additionalProperties": { "type": "string" }
                        },
                        "creationTimestamp": { "type": "string", "format": "date-time" }
                    }
                },
                "PodSpec": {
                    "type": "object",
                    "properties": {
                        "containers": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/Container" }
                        },
                        "restartPolicy": { "type": "string", "enum": ["Always", "OnFailure", "Never"] },
                        "nodeName": { "type": "string" }
                    }
                },
                "PodStatus": {
                    "type": "object",
                    "properties": {
                        "phase": { "type": "string", "enum": ["Pending", "Running", "Succeeded", "Failed", "Unknown"] },
                        "conditions": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/PodCondition" }
                        },
                        "containerStatuses": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/ContainerStatus" }
                        }
                    }
                },
                "Container": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "image": { "type": "string" },
                        "ports": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/ContainerPort" }
                        },
                        "env": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/EnvVar" }
                        }
                    }
                },
                "ContainerPort": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "containerPort": { "type": "integer" },
                        "protocol": { "type": "string", "enum": ["TCP", "UDP"] }
                    }
                },
                "EnvVar": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "value": { "type": "string" }
                    }
                },
                "PodCondition": {
                    "type": "object",
                    "properties": {
                        "type": { "type": "string" },
                        "status": { "type": "string", "enum": ["True", "False", "Unknown"] },
                        "lastTransitionTime": { "type": "string", "format": "date-time" },
                        "reason": { "type": "string" },
                        "message": { "type": "string" }
                    }
                },
                "ContainerStatus": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "state": { "$ref": "#/components/schemas/ContainerState" },
                        "ready": { "type": "boolean" },
                        "restartCount": { "type": "integer" }
                    }
                },
                "ContainerState": {
                    "type": "object",
                    "properties": {
                        "waiting": { "$ref": "#/components/schemas/ContainerStateWaiting" },
                        "running": { "$ref": "#/components/schemas/ContainerStateRunning" },
                        "terminated": { "$ref": "#/components/schemas/ContainerStateTerminated" }
                    }
                },
                "ContainerStateWaiting": {
                    "type": "object",
                    "properties": {
                        "reason": { "type": "string" },
                        "message": { "type": "string" }
                    }
                },
                "ContainerStateRunning": {
                    "type": "object",
                    "properties": {
                        "startedAt": { "type": "string", "format": "date-time" }
                    }
                },
                "ContainerStateTerminated": {
                    "type": "object",
                    "properties": {
                        "exitCode": { "type": "integer" },
                        "signal": { "type": "integer" },
                        "reason": { "type": "string" },
                        "message": { "type": "string" },
                        "startedAt": { "type": "string", "format": "date-time" },
                        "finishedAt": { "type": "string", "format": "date-time" }
                    }
                },
                "ServiceSpec": {
                    "type": "object",
                    "properties": {
                        "selector": {
                            "type": "object",
                            "additionalProperties": { "type": "string" }
                        },
                        "ports": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/ServicePort" }
                        },
                        "type": { "type": "string", "enum": ["ClusterIP", "NodePort", "LoadBalancer"] }
                    }
                },
                "ServicePort": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "port": { "type": "integer" },
                        "targetPort": { "type": "integer" },
                        "protocol": { "type": "string", "enum": ["TCP", "UDP"] }
                    }
                },
                "ServiceStatus": {
                    "type": "object",
                    "properties": {
                        "loadBalancer": { "$ref": "#/components/schemas/LoadBalancerStatus" }
                    }
                },
                "LoadBalancerStatus": {
                    "type": "object",
                    "properties": {
                        "ingress": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/LoadBalancerIngress" }
                        }
                    }
                },
                "LoadBalancerIngress": {
                    "type": "object",
                    "properties": {
                        "ip": { "type": "string" },
                        "hostname": { "type": "string" }
                    }
                }
            }
        }),
    };
    
    Json(spec)
}