use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Persistent volume management for container storage orchestration
#[derive(Debug, Clone)]
pub struct PersistentVolumeManager {
    pub volumes: Arc<Mutex<HashMap<String, PersistentVolume>>>,
    pub claims: Arc<Mutex<HashMap<String, PersistentVolumeClaim>>>,
    pub storage_classes: Arc<Mutex<HashMap<String, StorageClass>>>,
    pub provider: StorageProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolume {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub spec: PersistentVolumeSpec,
    pub status: PersistentVolumeStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeSpec {
    pub capacity: VolumeCapacity,
    pub access_modes: Vec<AccessMode>,
    pub persistent_volume_reclaim_policy: ReclaimPolicy,
    pub storage_class_name: Option<String>,
    pub mount_options: Vec<String>,
    pub volume_source: VolumeSource,
    pub node_affinity: Option<VolumeNodeAffinity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeCapacity {
    pub storage: String, // e.g., "10Gi", "1Ti"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessMode {
    ReadWriteOnce,
    ReadOnlyMany,
    ReadWriteMany,
    ReadWriteOncePod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReclaimPolicy {
    Retain,
    Recycle,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeSource {
    Local { path: String },
    AwsEbs { volume_id: String, fs_type: String },
    GcePd { pd_name: String, fs_type: String },
    AzureDisk { disk_name: String, disk_uri: String },
    Nfs { server: String, path: String },
    HostPath { path: String, host_path_type: HostPathType },
    EmptyDir { size_limit: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostPathType {
    Unset,
    DirectoryOrCreate,
    Directory,
    FileOrCreate,
    File,
    Socket,
    CharDevice,
    BlockDevice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeNodeAffinity {
    pub required: Option<NodeSelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelector {
    pub node_selector_terms: Vec<NodeSelectorTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorTerm {
    pub match_expressions: Vec<NodeSelectorRequirement>,
    pub match_fields: Vec<NodeSelectorRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorRequirement {
    pub key: String,
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeStatus {
    pub phase: VolumePhase,
    pub message: Option<String>,
    pub reason: Option<String>,
    pub last_phase_transition_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumePhase {
    Pending,
    Available,
    Bound,
    Released,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaim {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub spec: PersistentVolumeClaimSpec,
    pub status: PersistentVolumeClaimStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimSpec {
    pub access_modes: Vec<AccessMode>,
    pub resources: ResourceRequirements,
    pub volume_name: Option<String>,
    pub storage_class_name: Option<String>,
    pub volume_mode: Option<VolumeMode>,
    pub selector: Option<LabelSelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub requests: HashMap<String, String>,
    pub limits: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeMode {
    Filesystem,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    pub match_labels: HashMap<String, String>,
    pub match_expressions: Vec<LabelSelectorRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    pub key: String,
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimStatus {
    pub phase: ClaimPhase,
    pub access_modes: Vec<AccessMode>,
    pub capacity: Option<VolumeCapacity>,
    pub conditions: Vec<PersistentVolumeClaimCondition>,
    pub allocated_resources: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimPhase {
    Pending,
    Bound,
    Lost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimCondition {
    pub condition_type: String,
    pub status: String,
    pub last_probe_time: Option<DateTime<Utc>>,
    pub last_transition_time: DateTime<Utc>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClass {
    pub id: String,
    pub name: String,
    pub provisioner: String,
    pub parameters: HashMap<String, String>,
    pub reclaim_policy: ReclaimPolicy,
    pub allow_volume_expansion: bool,
    pub volume_binding_mode: VolumeBindingMode,
    pub allowed_topologies: Vec<TopologySelectorTerm>,
    pub mount_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeBindingMode {
    Immediate,
    WaitForFirstConsumer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySelectorTerm {
    pub match_label_expressions: Vec<TopologySelectorLabelRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySelectorLabelRequirement {
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum StorageProvider {
    Local(LocalStorageProvider),
    Aws(AwsStorageProvider),
    Gcp(GcpStorageProvider),
    Azure(AzureStorageProvider),
}

#[derive(Debug, Clone)]
pub struct LocalStorageProvider {
    pub base_path: String,
    pub max_size: u64, // in bytes
}

#[derive(Debug, Clone)]
pub struct AwsStorageProvider {
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct GcpStorageProvider {
    pub project_id: String,
    pub zone: String,
    pub credentials_path: String,
}

#[derive(Debug, Clone)]
pub struct AzureStorageProvider {
    pub subscription_id: String,
    pub resource_group: String,
    pub location: String,
}

impl PersistentVolumeManager {
    pub fn new(provider: StorageProvider) -> Self {
        let mut manager = Self {
            volumes: Arc::new(Mutex::new(HashMap::new())),
            claims: Arc::new(Mutex::new(HashMap::new())),
            storage_classes: Arc::new(Mutex::new(HashMap::new())),
            provider,
        };

        // Create default storage classes
        manager.create_default_storage_classes();
        manager
    }

    fn create_default_storage_classes(&mut self) {
        let mut storage_classes = self.storage_classes.lock().unwrap();

        // Local storage class
        let local_storage_class = StorageClass {
            id: Uuid::new_v4().to_string(),
            name: "local-storage".to_string(),
            provisioner: "infralink.io/local".to_string(),
            parameters: HashMap::new(),
            reclaim_policy: ReclaimPolicy::Delete,
            allow_volume_expansion: true,
            volume_binding_mode: VolumeBindingMode::WaitForFirstConsumer,
            allowed_topologies: vec![],
            mount_options: vec![],
        };
        storage_classes.insert("local-storage".to_string(), local_storage_class);

        // Fast SSD storage class
        let mut fast_params = HashMap::new();
        fast_params.insert("type".to_string(), "ssd".to_string());
        fast_params.insert("iops".to_string(), "3000".to_string());

        let fast_storage_class = StorageClass {
            id: Uuid::new_v4().to_string(),
            name: "fast-ssd".to_string(),
            provisioner: "infralink.io/cloud".to_string(),
            parameters: fast_params,
            reclaim_policy: ReclaimPolicy::Delete,
            allow_volume_expansion: true,
            volume_binding_mode: VolumeBindingMode::Immediate,
            allowed_topologies: vec![],
            mount_options: vec![],
        };
        storage_classes.insert("fast-ssd".to_string(), fast_storage_class);

        println!("Created default storage classes: local-storage, fast-ssd");
    }

    /// Create a persistent volume
    pub fn create_persistent_volume(
        &self,
        name: String,
        namespace: String,
        spec: PersistentVolumeSpec,
    ) -> Result<PersistentVolume, StorageError> {
        let volume_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Validate the volume specification
        self.validate_volume_spec(&spec)?;

        // Create the volume based on provider
        self.provision_volume(&spec)?;

        let volume = PersistentVolume {
            id: volume_id.clone(),
            name: name.clone(),
            namespace,
            spec,
            status: PersistentVolumeStatus {
                phase: VolumePhase::Available,
                message: Some("Volume successfully created".to_string()),
                reason: Some("VolumeCreated".to_string()),
                last_phase_transition_time: now,
            },
            created_at: now,
            updated_at: now,
        };

        let mut volumes = self.volumes.lock().unwrap();
        volumes.insert(volume_id, volume.clone());

        println!("Created persistent volume: {} ({})", name, volume.id);
        Ok(volume)
    }

    /// Create a persistent volume claim
    pub fn create_persistent_volume_claim(
        &self,
        name: String,
        namespace: String,
        spec: PersistentVolumeClaimSpec,
    ) -> Result<PersistentVolumeClaim, StorageError> {
        let claim_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Try to bind to an available volume
        let bound_volume = self.find_matching_volume(&spec)?;

        let (phase, conditions) = if bound_volume.is_some() {
            (ClaimPhase::Bound, vec![PersistentVolumeClaimCondition {
                condition_type: "Bound".to_string(),
                status: "True".to_string(),
                last_probe_time: Some(now),
                last_transition_time: now,
                reason: Some("VolumeBound".to_string()),
                message: Some("Volume successfully bound".to_string()),
            }])
        } else {
            (ClaimPhase::Pending, vec![PersistentVolumeClaimCondition {
                condition_type: "Pending".to_string(),
                status: "True".to_string(),
                last_probe_time: Some(now),
                last_transition_time: now,
                reason: Some("WaitingForVolume".to_string()),
                message: Some("Waiting for available volume".to_string()),
            }])
        };

        let capacity = bound_volume.as_ref().map(|v| v.spec.capacity.clone());

        let claim = PersistentVolumeClaim {
            id: claim_id.clone(),
            name: name.clone(),
            namespace,
            spec,
            status: PersistentVolumeClaimStatus {
                phase,
                access_modes: bound_volume.as_ref()
                    .map(|v| v.spec.access_modes.clone())
                    .unwrap_or_default(),
                capacity,
                conditions,
                allocated_resources: None,
            },
            created_at: now,
            updated_at: now,
        };

        // Update volume status if bound
        if let Some(volume) = bound_volume {
            self.bind_volume_to_claim(&volume.id, &claim_id)?;
        }

        let mut claims = self.claims.lock().unwrap();
        claims.insert(claim_id, claim.clone());

        println!("Created persistent volume claim: {} ({})", name, claim.id);
        Ok(claim)
    }

    /// Delete a persistent volume
    pub fn delete_persistent_volume(&self, volume_id: &str) -> Result<(), StorageError> {
        let volume = {
            let volumes = self.volumes.lock().unwrap();
            volumes.get(volume_id).cloned()
        };
        
        if let Some(volume) = volume {
            // Check if volume is bound
            if matches!(volume.status.phase, VolumePhase::Bound) {
                return Err(StorageError::VolumeInUse(volume_id.to_string()));
            }

            // Handle reclaim policy
            match volume.spec.persistent_volume_reclaim_policy {
                ReclaimPolicy::Delete => {
                    self.delete_volume_storage(&volume.spec.volume_source)?;
                }
                ReclaimPolicy::Retain => {
                    println!("Volume {} retained (not deleted from storage)", volume_id);
                }
                ReclaimPolicy::Recycle => {
                    self.recycle_volume_storage(&volume.spec.volume_source)?;
                }
            }

            let mut volumes = self.volumes.lock().unwrap();
            volumes.remove(volume_id);
            println!("Deleted persistent volume: {}", volume.name);
            Ok(())
        } else {
            Err(StorageError::VolumeNotFound(volume_id.to_string()))
        }
    }

    /// Delete a persistent volume claim
    pub fn delete_persistent_volume_claim(&self, claim_id: &str) -> Result<(), StorageError> {
        let mut claims = self.claims.lock().unwrap();
        
        if let Some(claim) = claims.remove(claim_id) {
            // Unbind volume if bound
            if matches!(claim.status.phase, ClaimPhase::Bound) {
                self.unbind_volume_from_claim(&claim)?;
            }

            println!("Deleted persistent volume claim: {}", claim.name);
            Ok(())
        } else {
            Err(StorageError::ClaimNotFound(claim_id.to_string()))
        }
    }

    /// List persistent volumes
    pub fn list_persistent_volumes(&self) -> Vec<PersistentVolume> {
        let volumes = self.volumes.lock().unwrap();
        volumes.values().cloned().collect()
    }

    /// List persistent volume claims
    pub fn list_persistent_volume_claims(&self, namespace: Option<&str>) -> Vec<PersistentVolumeClaim> {
        let claims = self.claims.lock().unwrap();
        claims
            .values()
            .filter(|claim| namespace.map_or(true, |ns| claim.namespace == ns))
            .cloned()
            .collect()
    }

    /// Get volume usage statistics
    pub fn get_storage_stats(&self) -> StorageStats {
        let volumes = self.volumes.lock().unwrap();
        let claims = self.claims.lock().unwrap();

        let mut total_capacity = 0u64;
        let mut allocated_capacity = 0u64;
        let mut volume_count_by_phase = HashMap::new();
        let mut claim_count_by_phase = HashMap::new();

        for volume in volumes.values() {
            if let Ok(capacity) = self.parse_capacity(&volume.spec.capacity.storage) {
                total_capacity += capacity;
                if matches!(volume.status.phase, VolumePhase::Bound) {
                    allocated_capacity += capacity;
                }
            }

            *volume_count_by_phase.entry(format!("{:?}", volume.status.phase)).or_insert(0) += 1;
        }

        for claim in claims.values() {
            *claim_count_by_phase.entry(format!("{:?}", claim.status.phase)).or_insert(0) += 1;
        }

        StorageStats {
            total_volumes: volumes.len(),
            total_claims: claims.len(),
            total_capacity,
            allocated_capacity,
            available_capacity: total_capacity - allocated_capacity,
            volume_count_by_phase,
            claim_count_by_phase,
        }
    }

    // Private helper methods

    fn validate_volume_spec(&self, spec: &PersistentVolumeSpec) -> Result<(), StorageError> {
        // Validate capacity format
        self.parse_capacity(&spec.capacity.storage)?;

        // Validate access modes
        if spec.access_modes.is_empty() {
            return Err(StorageError::InvalidConfiguration("No access modes specified".to_string()));
        }

        // Validate storage class exists
        if let Some(storage_class_name) = &spec.storage_class_name {
            let storage_classes = self.storage_classes.lock().unwrap();
            if !storage_classes.contains_key(storage_class_name) {
                return Err(StorageError::StorageClassNotFound(storage_class_name.clone()));
            }
        }

        Ok(())
    }

    fn provision_volume(&self, spec: &PersistentVolumeSpec) -> Result<(), StorageError> {
        match &self.provider {
            StorageProvider::Local(local) => {
                self.provision_local_volume(local, spec)
            }
            StorageProvider::Aws(aws) => {
                self.provision_aws_volume(aws, spec)
            }
            StorageProvider::Gcp(gcp) => {
                self.provision_gcp_volume(gcp, spec)
            }
            StorageProvider::Azure(azure) => {
                self.provision_azure_volume(azure, spec)
            }
        }
    }

    fn provision_local_volume(&self, _provider: &LocalStorageProvider, spec: &PersistentVolumeSpec) -> Result<(), StorageError> {
        if let VolumeSource::Local { path } = &spec.volume_source {
            // Create directory if it doesn't exist
            std::fs::create_dir_all(path)
                .map_err(|e| StorageError::ProvisioningFailed(format!("Failed to create directory {}: {}", path, e)))?;
            
            println!("Provisioned local volume at: {}", path);
        } else if let VolumeSource::HostPath { path, .. } = &spec.volume_source {
            // Validate host path exists
            if !std::path::Path::new(path).exists() {
                return Err(StorageError::ProvisioningFailed(format!("Host path does not exist: {}", path)));
            }
            
            println!("Validated host path volume at: {}", path);
        }
        
        Ok(())
    }

    fn provision_aws_volume(&self, _provider: &AwsStorageProvider, _spec: &PersistentVolumeSpec) -> Result<(), StorageError> {
        // Mock AWS EBS volume creation
        println!("Mock: Created AWS EBS volume");
        Ok(())
    }

    fn provision_gcp_volume(&self, _provider: &GcpStorageProvider, _spec: &PersistentVolumeSpec) -> Result<(), StorageError> {
        // Mock GCP Persistent Disk creation
        println!("Mock: Created GCP Persistent Disk");
        Ok(())
    }

    fn provision_azure_volume(&self, _provider: &AzureStorageProvider, _spec: &PersistentVolumeSpec) -> Result<(), StorageError> {
        // Mock Azure Disk creation
        println!("Mock: Created Azure Disk");
        Ok(())
    }

    fn find_matching_volume(&self, claim_spec: &PersistentVolumeClaimSpec) -> Result<Option<PersistentVolume>, StorageError> {
        let volumes = self.volumes.lock().unwrap();
        
        for volume in volumes.values() {
            if matches!(volume.status.phase, VolumePhase::Available) {
                // Check access modes compatibility
                let access_modes_match = claim_spec.access_modes.iter()
                    .all(|mode| volume.spec.access_modes.contains(mode));

                // Check storage capacity
                let capacity_match = if let Some(requested) = claim_spec.resources.requests.get("storage") {
                    let volume_capacity = self.parse_capacity(&volume.spec.capacity.storage)?;
                    let requested_capacity = self.parse_capacity(requested)?;
                    volume_capacity >= requested_capacity
                } else {
                    true
                };

                // Check storage class
                let storage_class_match = match (&claim_spec.storage_class_name, &volume.spec.storage_class_name) {
                    (Some(claim_class), Some(volume_class)) => claim_class == volume_class,
                    (None, None) => true,
                    _ => false,
                };

                if access_modes_match && capacity_match && storage_class_match {
                    return Ok(Some(volume.clone()));
                }
            }
        }

        Ok(None)
    }

    fn bind_volume_to_claim(&self, volume_id: &str, claim_id: &str) -> Result<(), StorageError> {
        let mut volumes = self.volumes.lock().unwrap();
        
        if let Some(volume) = volumes.get_mut(volume_id) {
            volume.status.phase = VolumePhase::Bound;
            volume.status.message = Some(format!("Bound to claim {}", claim_id));
            volume.status.last_phase_transition_time = Utc::now();
            volume.updated_at = Utc::now();
            
            println!("Bound volume {} to claim {}", volume_id, claim_id);
            Ok(())
        } else {
            Err(StorageError::VolumeNotFound(volume_id.to_string()))
        }
    }

    fn unbind_volume_from_claim(&self, claim: &PersistentVolumeClaim) -> Result<(), StorageError> {
        let mut volumes = self.volumes.lock().unwrap();
        
        // Find volume bound to this claim
        for volume in volumes.values_mut() {
            if matches!(volume.status.phase, VolumePhase::Bound) {
                if let Some(message) = &volume.status.message {
                    if message.contains(&claim.id) {
                        volume.status.phase = VolumePhase::Released;
                        volume.status.message = Some("Released from claim".to_string());
                        volume.status.last_phase_transition_time = Utc::now();
                        volume.updated_at = Utc::now();
                        
                        println!("Unbound volume {} from claim {}", volume.id, claim.id);
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    fn delete_volume_storage(&self, volume_source: &VolumeSource) -> Result<(), StorageError> {
        match volume_source {
            VolumeSource::Local { path } | VolumeSource::HostPath { path, .. } => {
                std::fs::remove_dir_all(path)
                    .map_err(|e| StorageError::DeletionFailed(format!("Failed to delete directory {}: {}", path, e)))?;
                println!("Deleted local volume storage at: {}", path);
            }
            _ => {
                println!("Mock: Deleted cloud volume storage");
            }
        }
        Ok(())
    }

    pub fn recycle_volume_storage(&self, volume_source: &VolumeSource) -> Result<(), StorageError> {
        match volume_source {
            VolumeSource::Local { path } | VolumeSource::HostPath { path, .. } => {
                // Clear directory contents but keep the directory
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
                println!("Recycled local volume storage at: {}", path);
            }
            _ => {
                println!("Mock: Recycled cloud volume storage");
            }
        }
        Ok(())
    }

    pub fn parse_capacity(&self, capacity_str: &str) -> Result<u64, StorageError> {
        let capacity_str = capacity_str.trim();
        
        if capacity_str.is_empty() {
            return Err(StorageError::InvalidCapacity("Empty capacity string".to_string()));
        }

        let (number_part, unit) = if capacity_str.ends_with("Ki") {
            (&capacity_str[..capacity_str.len()-2], 1024u64)
        } else if capacity_str.ends_with("Mi") {
            (&capacity_str[..capacity_str.len()-2], 1024u64.pow(2))
        } else if capacity_str.ends_with("Gi") {
            (&capacity_str[..capacity_str.len()-2], 1024u64.pow(3))
        } else if capacity_str.ends_with("Ti") {
            (&capacity_str[..capacity_str.len()-2], 1024u64.pow(4))
        } else if capacity_str.ends_with("Pi") {
            (&capacity_str[..capacity_str.len()-2], 1024u64.pow(5))
        } else if capacity_str.ends_with('B') {
            (&capacity_str[..capacity_str.len()-1], 1)
        } else {
            // Assume bytes if no unit
            (capacity_str, 1)
        };

        let number: u64 = number_part.parse()
            .map_err(|_| StorageError::InvalidCapacity(format!("Invalid number: {}", number_part)))?;

        Ok(number * unit)
    }
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_volumes: usize,
    pub total_claims: usize,
    pub total_capacity: u64,
    pub allocated_capacity: u64,
    pub available_capacity: u64,
    pub volume_count_by_phase: HashMap<String, usize>,
    pub claim_count_by_phase: HashMap<String, usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Volume not found: {0}")]
    VolumeNotFound(String),
    #[error("Claim not found: {0}")]
    ClaimNotFound(String),
    #[error("Storage class not found: {0}")]
    StorageClassNotFound(String),
    #[error("Volume is in use: {0}")]
    VolumeInUse(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Invalid capacity format: {0}")]
    InvalidCapacity(String),
    #[error("Provisioning failed: {0}")]
    ProvisioningFailed(String),
    #[error("Deletion failed: {0}")]
    DeletionFailed(String),
    #[error("Binding failed: {0}")]
    BindingFailed(String),
}

impl Default for StorageProvider {
    fn default() -> Self {
        StorageProvider::Local(LocalStorageProvider {
            base_path: "/tmp/infralink/volumes".to_string(),
            max_size: 100 * 1024 * 1024 * 1024, // 100GB
        })
    }
}