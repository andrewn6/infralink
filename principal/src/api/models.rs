use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Common Kubernetes-compatible data structures for the API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub name: String,
    pub namespace: Option<String>,
    pub uid: Option<String>,
    pub resource_version: Option<String>,
    pub generation: Option<i64>,
    pub creation_timestamp: Option<String>,
    pub deletion_timestamp: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    pub annotations: Option<HashMap<String, String>>,
    pub owner_references: Option<Vec<OwnerReference>>,
    pub finalizers: Option<Vec<String>>,
}

impl Default for ObjectMeta {
    fn default() -> Self {
        Self {
            name: String::new(),
            namespace: None,
            uid: None,
            resource_version: None,
            generation: None,
            creation_timestamp: None,
            deletion_timestamp: None,
            labels: None,
            annotations: None,
            owner_references: None,
            finalizers: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerReference {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub uid: String,
    pub controller: Option<bool>,
    pub block_owner_deletion: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMeta {
    pub kind: String,
    pub api_version: String,
}

impl Default for TypeMeta {
    fn default() -> Self {
        Self {
            kind: String::new(),
            api_version: "v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMeta {
    pub resource_version: Option<String>,
    pub continue_token: Option<String>,
    pub remaining_item_count: Option<i64>,
}

/// Label selector for filtering resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    pub match_labels: Option<HashMap<String, String>>,
    pub match_expressions: Option<Vec<LabelSelectorRequirement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    pub key: String,
    pub operator: String, // In, NotIn, Exists, DoesNotExist
    pub values: Option<Vec<String>>,
}

/// Resource requests and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub limits: Option<HashMap<String, String>>,
    pub requests: Option<HashMap<String, String>>,
}

/// Condition represents a single condition of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String, // True, False, Unknown
    pub last_transition_time: Option<String>,
    pub last_update_time: Option<String>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Generic list wrapper for Kubernetes resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List<T> {
    pub type_meta: TypeMeta,
    pub list_meta: Option<ListMeta>,
    pub items: Vec<T>,
}

impl<T> List<T> {
    pub fn new(kind: &str, items: Vec<T>) -> Self {
        Self {
            type_meta: TypeMeta {
                kind: format!("{}List", kind),
                api_version: "v1".to_string(),
            },
            list_meta: None,
            items,
        }
    }
}

/// Quantity represents a fixed-point number with a scale factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    pub value: String,
}

impl Quantity {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

/// Local object reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalObjectReference {
    pub name: String,
}

/// Typed local object reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedLocalObjectReference {
    pub api_version: Option<String>,
    pub kind: String,
    pub name: String,
}

/// Volume source for different types of volumes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSource {
    pub host_path: Option<HostPathVolumeSource>,
    pub empty_dir: Option<EmptyDirVolumeSource>,
    pub persistent_volume_claim: Option<PersistentVolumeClaimVolumeSource>,
    pub config_map: Option<ConfigMapVolumeSource>,
    pub secret: Option<SecretVolumeSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostPathVolumeSource {
    pub path: String,
    #[serde(rename = "type")]
    pub path_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyDirVolumeSource {
    pub medium: Option<String>,
    pub size_limit: Option<Quantity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimVolumeSource {
    pub claim_name: String,
    pub read_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapVolumeSource {
    pub name: String,
    pub default_mode: Option<i32>,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretVolumeSource {
    pub secret_name: String,
    pub default_mode: Option<i32>,
    pub optional: Option<bool>,
}

/// Volume mount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    pub sub_path: Option<String>,
    pub read_only: Option<bool>,
}

/// Environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: Option<String>,
    pub value_from: Option<EnvVarSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarSource {
    pub field_ref: Option<ObjectFieldSelector>,
    pub resource_field_ref: Option<ResourceFieldSelector>,
    pub config_map_key_ref: Option<ConfigMapKeySelector>,
    pub secret_key_ref: Option<SecretKeySelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectFieldSelector {
    pub api_version: Option<String>,
    pub field_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFieldSelector {
    pub container_name: Option<String>,
    pub resource: String,
    pub divisor: Option<Quantity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapKeySelector {
    pub name: String,
    pub key: String,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretKeySelector {
    pub name: String,
    pub key: String,
    pub optional: Option<bool>,
}