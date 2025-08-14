use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};

/// Configuration management for secrets and config maps
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// ConfigMaps storage
    config_maps: HashMap<String, ConfigMap>,
    /// Secrets storage (encrypted)
    secrets: HashMap<String, Secret>,
    /// Mounted configurations per pod
    pod_mounts: HashMap<String, Vec<ConfigMount>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMap {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub data: HashMap<String, String>,
    pub binary_data: HashMap<String, Vec<u8>>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub secret_type: SecretType,
    pub data: HashMap<String, String>, // Base64 encoded values
    pub string_data: HashMap<String, String>, // Plain text values (will be encoded)
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretType {
    Opaque,
    ServiceAccountToken,
    DockerConfigJson,
    DockerConfig,
    BasicAuth,
    SSHAuth,
    TLS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMount {
    pub config_name: String,
    pub config_type: ConfigType,
    pub mount_path: String,
    pub sub_path: Option<String>,
    pub read_only: bool,
    pub file_mode: Option<u32>,
    pub mount_type: ConfigMountType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigType {
    ConfigMap,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigMountType {
    Volume,
    Environment,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapCreate {
    pub name: String,
    pub namespace: String,
    pub data: HashMap<String, String>,
    pub binary_data: Option<HashMap<String, Vec<u8>>>,
    pub labels: Option<HashMap<String, String>>,
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretCreate {
    pub name: String,
    pub namespace: String,
    pub secret_type: SecretType,
    pub data: Option<HashMap<String, String>>, // Base64 encoded
    pub string_data: Option<HashMap<String, String>>, // Plain text
    pub labels: Option<HashMap<String, String>>,
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResolution {
    pub environment_variables: HashMap<String, String>,
    pub volume_mounts: Vec<VolumeMount>,
    pub files: HashMap<String, FileContent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    pub read_only: bool,
    pub file_mode: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub content: String,
    pub binary: bool,
    pub mode: Option<u32>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config_maps: HashMap::new(),
            secrets: HashMap::new(),
            pod_mounts: HashMap::new(),
        }
    }

    /// Create a new ConfigMap
    pub fn create_config_map(&mut self, request: ConfigMapCreate) -> Result<String, Box<dyn std::error::Error>> {
        let config_map_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let config_map = ConfigMap {
            id: config_map_id.clone(),
            name: request.name.clone(),
            namespace: request.namespace.clone(),
            data: request.data,
            binary_data: request.binary_data.unwrap_or_default(),
            labels: request.labels.unwrap_or_default(),
            annotations: request.annotations.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        let key = self.config_key(&request.name, &request.namespace);
        self.config_maps.insert(key, config_map);

        println!("Created ConfigMap {} in namespace {}", request.name, request.namespace);
        Ok(config_map_id)
    }

    /// Create a new Secret
    pub fn create_secret(&mut self, request: SecretCreate) -> Result<String, Box<dyn std::error::Error>> {
        let secret_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        // Encode string_data to base64
        let mut data = request.data.unwrap_or_default();
        if let Some(string_data) = request.string_data {
            for (key, value) in string_data {
                let encoded = general_purpose::STANDARD.encode(value.as_bytes());
                data.insert(key, encoded);
            }
        }

        let secret = Secret {
            id: secret_id.clone(),
            name: request.name.clone(),
            namespace: request.namespace.clone(),
            secret_type: request.secret_type,
            data,
            string_data: HashMap::new(), // Clear after encoding
            labels: request.labels.unwrap_or_default(),
            annotations: request.annotations.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        let key = self.config_key(&request.name, &request.namespace);
        self.secrets.insert(key, secret);

        println!("Created Secret {} in namespace {}", request.name, request.namespace);
        Ok(secret_id)
    }

    /// Get a ConfigMap
    pub fn get_config_map(&self, name: &str, namespace: &str) -> Option<&ConfigMap> {
        let key = self.config_key(name, namespace);
        self.config_maps.get(&key)
    }

    /// Get a Secret
    pub fn get_secret(&self, name: &str, namespace: &str) -> Option<&Secret> {
        let key = self.config_key(name, namespace);
        self.secrets.get(&key)
    }

    /// Update a ConfigMap
    pub fn update_config_map(&mut self, name: &str, namespace: &str, data: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.config_key(name, namespace);
        
        if let Some(config_map) = self.config_maps.get_mut(&key) {
            config_map.data = data;
            config_map.updated_at = Utc::now();
            println!("Updated ConfigMap {} in namespace {}", name, namespace);
            Ok(())
        } else {
            Err(format!("ConfigMap {} not found in namespace {}", name, namespace).into())
        }
    }

    /// Update a Secret
    pub fn update_secret(&mut self, name: &str, namespace: &str, string_data: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.config_key(name, namespace);
        
        if let Some(secret) = self.secrets.get_mut(&key) {
            // Encode new data
            let mut encoded_data = HashMap::new();
            for (k, v) in string_data {
                let encoded = general_purpose::STANDARD.encode(v.as_bytes());
                encoded_data.insert(k, encoded);
            }
            
            secret.data = encoded_data;
            secret.updated_at = Utc::now();
            println!("Updated Secret {} in namespace {}", name, namespace);
            Ok(())
        } else {
            Err(format!("Secret {} not found in namespace {}", name, namespace).into())
        }
    }

    /// Delete a ConfigMap
    pub fn delete_config_map(&mut self, name: &str, namespace: &str) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.config_key(name, namespace);
        
        if self.config_maps.remove(&key).is_some() {
            println!("Deleted ConfigMap {} from namespace {}", name, namespace);
            Ok(())
        } else {
            Err(format!("ConfigMap {} not found in namespace {}", name, namespace).into())
        }
    }

    /// Delete a Secret
    pub fn delete_secret(&mut self, name: &str, namespace: &str) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.config_key(name, namespace);
        
        if self.secrets.remove(&key).is_some() {
            println!("Deleted Secret {} from namespace {}", name, namespace);
            Ok(())
        } else {
            Err(format!("Secret {} not found in namespace {}", name, namespace).into())
        }
    }

    /// Mount configuration to a pod
    pub fn mount_config_to_pod(&mut self, pod_id: &str, mount: ConfigMount) -> Result<(), Box<dyn std::error::Error>> {
        // Validate that the config exists
        match mount.config_type {
            ConfigType::ConfigMap => {
                let key = self.config_key(&mount.config_name, "default"); // TODO: Use proper namespace
                if !self.config_maps.contains_key(&key) {
                    return Err(format!("ConfigMap {} not found", mount.config_name).into());
                }
            }
            ConfigType::Secret => {
                let key = self.config_key(&mount.config_name, "default");
                if !self.secrets.contains_key(&key) {
                    return Err(format!("Secret {} not found", mount.config_name).into());
                }
            }
        }

        self.pod_mounts.entry(pod_id.to_string()).or_insert_with(Vec::new).push(mount);
        Ok(())
    }

    /// Resolve all configurations for a pod
    pub fn resolve_pod_config(&self, pod_id: &str, namespace: &str) -> Result<ConfigResolution, Box<dyn std::error::Error>> {
        let mut environment_variables = HashMap::new();
        let mut volume_mounts = Vec::new();
        let mut files = HashMap::new();

        if let Some(mounts) = self.pod_mounts.get(pod_id) {
            for mount in mounts {
                match mount.config_type {
                    ConfigType::ConfigMap => {
                        if let Some(config_map) = self.get_config_map(&mount.config_name, namespace) {
                            self.apply_config_map_mount(config_map, mount, &mut environment_variables, &mut volume_mounts, &mut files)?;
                        }
                    }
                    ConfigType::Secret => {
                        if let Some(secret) = self.get_secret(&mount.config_name, namespace) {
                            self.apply_secret_mount(secret, mount, &mut environment_variables, &mut volume_mounts, &mut files)?;
                        }
                    }
                }
            }
        }

        Ok(ConfigResolution {
            environment_variables,
            volume_mounts,
            files,
        })
    }

    fn apply_config_map_mount(
        &self,
        config_map: &ConfigMap,
        mount: &ConfigMount,
        env_vars: &mut HashMap<String, String>,
        volume_mounts: &mut Vec<VolumeMount>,
        files: &mut HashMap<String, FileContent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match mount.mount_type {
            ConfigMountType::Environment => {
                // Mount all keys as environment variables
                for (key, value) in &config_map.data {
                    let env_key = if let Some(prefix) = mount.sub_path.as_ref() {
                        format!("{}_{}", prefix.to_uppercase(), key.to_uppercase())
                    } else {
                        key.to_uppercase()
                    };
                    env_vars.insert(env_key, value.clone());
                }
            }
            ConfigMountType::Volume => {
                volume_mounts.push(VolumeMount {
                    name: config_map.name.clone(),
                    mount_path: mount.mount_path.clone(),
                    read_only: mount.read_only,
                    file_mode: mount.file_mode,
                });

                // Create files for each key
                for (key, value) in &config_map.data {
                    let file_path = if let Some(sub_path) = &mount.sub_path {
                        format!("{}/{}", mount.mount_path, sub_path)
                    } else {
                        format!("{}/{}", mount.mount_path, key)
                    };

                    files.insert(file_path, FileContent {
                        content: value.clone(),
                        binary: false,
                        mode: mount.file_mode,
                    });
                }

                // Handle binary data
                for (key, value) in &config_map.binary_data {
                    let file_path = format!("{}/{}", mount.mount_path, key);
                    let content = general_purpose::STANDARD.encode(value);

                    files.insert(file_path, FileContent {
                        content,
                        binary: true,
                        mode: mount.file_mode,
                    });
                }
            }
            ConfigMountType::File => {
                if let Some(sub_path) = &mount.sub_path {
                    if let Some(value) = config_map.data.get(sub_path) {
                        files.insert(mount.mount_path.clone(), FileContent {
                            content: value.clone(),
                            binary: false,
                            mode: mount.file_mode,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn apply_secret_mount(
        &self,
        secret: &Secret,
        mount: &ConfigMount,
        env_vars: &mut HashMap<String, String>,
        volume_mounts: &mut Vec<VolumeMount>,
        files: &mut HashMap<String, FileContent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match mount.mount_type {
            ConfigMountType::Environment => {
                // Decode and mount as environment variables
                for (key, encoded_value) in &secret.data {
                    let decoded = general_purpose::STANDARD.decode(encoded_value)
                        .map_err(|e| format!("Failed to decode secret value: {}", e))?;
                    let value = String::from_utf8(decoded)
                        .map_err(|e| format!("Invalid UTF-8 in secret value: {}", e))?;

                    let env_key = if let Some(prefix) = mount.sub_path.as_ref() {
                        format!("{}_{}", prefix.to_uppercase(), key.to_uppercase())
                    } else {
                        key.to_uppercase()
                    };
                    env_vars.insert(env_key, value);
                }
            }
            ConfigMountType::Volume => {
                volume_mounts.push(VolumeMount {
                    name: secret.name.clone(),
                    mount_path: mount.mount_path.clone(),
                    read_only: mount.read_only,
                    file_mode: mount.file_mode,
                });

                // Create files for each key (decoded)
                for (key, encoded_value) in &secret.data {
                    let decoded = general_purpose::STANDARD.decode(encoded_value)
                        .map_err(|e| format!("Failed to decode secret value: {}", e))?;
                    let content = String::from_utf8(decoded)
                        .map_err(|e| format!("Invalid UTF-8 in secret value: {}", e))?;

                    let file_path = if let Some(sub_path) = &mount.sub_path {
                        format!("{}/{}", mount.mount_path, sub_path)
                    } else {
                        format!("{}/{}", mount.mount_path, key)
                    };

                    files.insert(file_path, FileContent {
                        content,
                        binary: false,
                        mode: mount.file_mode.or(Some(0o600)), // Secrets default to 600
                    });
                }
            }
            ConfigMountType::File => {
                if let Some(sub_path) = &mount.sub_path {
                    if let Some(encoded_value) = secret.data.get(sub_path) {
                        let decoded = general_purpose::STANDARD.decode(encoded_value)
                            .map_err(|e| format!("Failed to decode secret value: {}", e))?;
                        let content = String::from_utf8(decoded)
                            .map_err(|e| format!("Invalid UTF-8 in secret value: {}", e))?;

                        files.insert(mount.mount_path.clone(), FileContent {
                            content,
                            binary: false,
                            mode: mount.file_mode.or(Some(0o600)),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// List ConfigMaps in a namespace
    pub fn list_config_maps(&self, namespace: &str) -> Vec<&ConfigMap> {
        self.config_maps.values()
            .filter(|cm| cm.namespace == namespace)
            .collect()
    }

    /// List Secrets in a namespace
    pub fn list_secrets(&self, namespace: &str) -> Vec<&Secret> {
        self.secrets.values()
            .filter(|s| s.namespace == namespace)
            .collect()
    }

    /// Get configuration statistics
    pub fn get_stats(&self) -> ConfigManagerStats {
        ConfigManagerStats {
            total_config_maps: self.config_maps.len(),
            total_secrets: self.secrets.len(),
            total_pod_mounts: self.pod_mounts.len(),
        }
    }

    fn config_key(&self, name: &str, namespace: &str) -> String {
        format!("{}.{}", name, namespace)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigManagerStats {
    pub total_config_maps: usize,
    pub total_secrets: usize,
    pub total_pod_mounts: usize,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}