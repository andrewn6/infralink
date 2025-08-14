use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use uuid::Uuid;
use chrono::Utc;

use crate::services::storage::{
    PersistentVolumeManager, PersistentVolumeClaim, PersistentVolume, StorageError,
    PersistentVolumeSpec, VolumeSource, ReclaimPolicy, VolumeCapacity,
    ClaimPhase, VolumePhase, PersistentVolumeClaimCondition, StorageProvider,
};

/// Volume scheduler for dynamic provisioning and lifecycle management
#[derive(Clone)]
pub struct VolumeScheduler {
    volume_manager: Arc<PersistentVolumeManager>,
    provisioning_queue: Arc<Mutex<Vec<String>>>, // claim IDs waiting for volumes
    recycling_queue: Arc<Mutex<Vec<String>>>,    // volume IDs to be recycled
    config: VolumeSchedulerConfig,
}

#[derive(Debug, Clone)]
pub struct VolumeSchedulerConfig {
    pub dynamic_provisioning_enabled: bool,
    pub auto_scaling_enabled: bool,
    pub recycling_enabled: bool,
    pub check_interval: Duration,
    pub max_pending_time: Duration,
    pub storage_efficiency_threshold: f64, // 0.0 - 1.0
}

impl Default for VolumeSchedulerConfig {
    fn default() -> Self {
        Self {
            dynamic_provisioning_enabled: true,
            auto_scaling_enabled: true,
            recycling_enabled: true,
            check_interval: Duration::from_secs(30),
            max_pending_time: Duration::from_secs(300), // 5 minutes
            storage_efficiency_threshold: 0.8, // 80% utilization before scaling
        }
    }
}

impl VolumeScheduler {
    pub fn new(volume_manager: Arc<PersistentVolumeManager>, config: VolumeSchedulerConfig) -> Self {
        Self {
            volume_manager,
            provisioning_queue: Arc::new(Mutex::new(Vec::new())),
            recycling_queue: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Start the volume scheduler background tasks
    pub async fn start(&self) {
        println!("Starting volume scheduler with config: {:?}", self.config);
        
        let scheduler = self.clone();
        tokio::spawn(async move {
            scheduler.run_scheduler_loop().await;
        });

        if self.config.recycling_enabled {
            let scheduler = self.clone();
            tokio::spawn(async move {
                scheduler.run_recycling_loop().await;
            });
        }
    }

    /// Handle a new persistent volume claim that needs provisioning
    pub fn enqueue_claim_for_provisioning(&self, claim_id: String) {
        let mut queue = self.provisioning_queue.lock().unwrap();
        if !queue.contains(&claim_id) {
            queue.push(claim_id.clone());
            println!("Enqueued claim {} for dynamic provisioning", claim_id);
        }
    }

    /// Handle a volume that needs recycling
    pub fn enqueue_volume_for_recycling(&self, volume_id: String) {
        let mut queue = self.recycling_queue.lock().unwrap();
        if !queue.contains(&volume_id) {
            queue.push(volume_id.clone());
            println!("Enqueued volume {} for recycling", volume_id);
        }
    }

    /// Main scheduler loop
    async fn run_scheduler_loop(&self) {
        let mut interval = interval(self.config.check_interval);

        loop {
            interval.tick().await;

            // Process dynamic provisioning
            if self.config.dynamic_provisioning_enabled {
                if let Err(e) = self.process_dynamic_provisioning().await {
                    eprintln!("Error in dynamic provisioning: {}", e);
                }
            }

            // Process volume binding
            if let Err(e) = self.process_volume_binding().await {
                eprintln!("Error in volume binding: {}", e);
            }

            // Check for orphaned resources
            if let Err(e) = self.cleanup_orphaned_resources().await {
                eprintln!("Error in orphaned resource cleanup: {}", e);
            }

            // Auto-scaling if enabled
            if self.config.auto_scaling_enabled {
                if let Err(e) = self.check_auto_scaling().await {
                    eprintln!("Error in auto-scaling check: {}", e);
                }
            }
        }
    }

    /// Recycling loop for released volumes
    async fn run_recycling_loop(&self) {
        let mut interval = interval(Duration::from_secs(60)); // Check every minute

        loop {
            interval.tick().await;

            if let Err(e) = self.process_recycling().await {
                eprintln!("Error in volume recycling: {}", e);
            }
        }
    }

    /// Process claims waiting for dynamic provisioning
    async fn process_dynamic_provisioning(&self) -> Result<(), StorageError> {
        let claim_ids: Vec<String> = {
            let mut queue = self.provisioning_queue.lock().unwrap();
            queue.drain(..).collect()
        };

        for claim_id in claim_ids {
            if let Err(e) = self.provision_volume_for_claim(&claim_id).await {
                eprintln!("Failed to provision volume for claim {}: {}", claim_id, e);
                
                // Re-queue if it's a temporary failure
                if matches!(e, StorageError::ProvisioningFailed(_)) {
                    let mut queue = self.provisioning_queue.lock().unwrap();
                    queue.push(claim_id);
                }
            }
        }

        Ok(())
    }

    /// Provision a new volume for a specific claim
    async fn provision_volume_for_claim(&self, claim_id: &str) -> Result<(), StorageError> {
        let claims = self.volume_manager.claims.lock().unwrap();
        let claim = claims.get(claim_id)
            .ok_or_else(|| StorageError::ClaimNotFound(claim_id.to_string()))?
            .clone();
        drop(claims);

        // Only provision for pending claims
        if !matches!(claim.status.phase, ClaimPhase::Pending) {
            return Ok(());
        }

        // Check if claim has been pending too long
        let pending_duration = Utc::now().signed_duration_since(claim.created_at);
        if pending_duration.to_std().unwrap_or(Duration::ZERO) > self.config.max_pending_time {
            println!("Claim {} has been pending too long, triggering dynamic provisioning", claim_id);
        }

        // Create volume spec based on claim requirements
        let volume_spec = self.create_volume_spec_from_claim(&claim)?;
        
        let volume_name = format!("pv-{}", Uuid::new_v4().to_string()[..8].to_lowercase());
        
        // Create the volume
        let volume = self.volume_manager.create_persistent_volume(
            volume_name,
            claim.namespace.clone(),
            volume_spec,
        )?;

        // Try to bind immediately
        if let Err(e) = self.bind_claim_to_volume(&claim.id, &volume.id) {
            eprintln!("Failed to bind claim {} to newly provisioned volume {}: {}", 
                     claim_id, volume.id, e);
        } else {
            println!("Successfully provisioned and bound volume {} for claim {}", 
                    volume.id, claim_id);
        }

        Ok(())
    }

    /// Create volume spec from claim requirements
    fn create_volume_spec_from_claim(&self, claim: &PersistentVolumeClaim) -> Result<PersistentVolumeSpec, StorageError> {
        // Determine storage size
        let storage_size = claim.spec.resources.requests
            .get("storage")
            .ok_or_else(|| StorageError::InvalidConfiguration("No storage size specified in claim".to_string()))?
            .clone();

        // Determine volume source based on storage class
        let volume_source = if let Some(storage_class_name) = &claim.spec.storage_class_name {
            match storage_class_name.as_str() {
                "local-storage" => VolumeSource::Local {
                    path: format!("/tmp/infralink/volumes/{}", Uuid::new_v4()),
                },
                "fast-ssd" => {
                    // For cloud providers, this would create appropriate volume type
                    match &self.volume_manager.provider {
                        StorageProvider::Local(_) => VolumeSource::Local {
                            path: format!("/tmp/infralink/volumes/ssd-{}", Uuid::new_v4()),
                        },
                        StorageProvider::Aws(_) => VolumeSource::AwsEbs {
                            volume_id: format!("vol-{}", Uuid::new_v4()),
                            fs_type: "ext4".to_string(),
                        },
                        StorageProvider::Gcp(_) => VolumeSource::GcePd {
                            pd_name: format!("pd-{}", Uuid::new_v4()),
                            fs_type: "ext4".to_string(),
                        },
                        StorageProvider::Azure(_) => VolumeSource::AzureDisk {
                            disk_name: format!("disk-{}", Uuid::new_v4()),
                            disk_uri: format!("https://example.disk.azure.com/{}", Uuid::new_v4()),
                        },
                    }
                },
                _ => VolumeSource::Local {
                    path: format!("/tmp/infralink/volumes/{}", Uuid::new_v4()),
                },
            }
        } else {
            VolumeSource::Local {
                path: format!("/tmp/infralink/volumes/{}", Uuid::new_v4()),
            }
        };

        Ok(PersistentVolumeSpec {
            capacity: VolumeCapacity {
                storage: storage_size,
            },
            access_modes: claim.spec.access_modes.clone(),
            persistent_volume_reclaim_policy: ReclaimPolicy::Delete,
            storage_class_name: claim.spec.storage_class_name.clone(),
            mount_options: vec![],
            volume_source,
            node_affinity: None,
        })
    }

    /// Process volume binding for unbound claims
    async fn process_volume_binding(&self) -> Result<(), StorageError> {
        let claims: Vec<PersistentVolumeClaim> = {
            let claims = self.volume_manager.claims.lock().unwrap();
            claims.values()
                .filter(|claim| matches!(claim.status.phase, ClaimPhase::Pending))
                .cloned()
                .collect()
        };

        for claim in claims {
            if let Some(volume) = self.find_available_volume_for_claim(&claim)? {
                if let Err(e) = self.bind_claim_to_volume(&claim.id, &volume.id) {
                    eprintln!("Failed to bind claim {} to volume {}: {}", claim.id, volume.id, e);
                } else {
                    println!("Bound existing volume {} to claim {}", volume.id, claim.id);
                }
            } else if self.config.dynamic_provisioning_enabled {
                // Queue for dynamic provisioning
                self.enqueue_claim_for_provisioning(claim.id.clone());
            }
        }

        Ok(())
    }

    /// Find an available volume that matches claim requirements
    fn find_available_volume_for_claim(&self, claim: &PersistentVolumeClaim) -> Result<Option<PersistentVolume>, StorageError> {
        let volumes = self.volume_manager.volumes.lock().unwrap();
        
        for volume in volumes.values() {
            if matches!(volume.status.phase, VolumePhase::Available) {
                // Check if volume satisfies claim requirements
                if self.volume_satisfies_claim(volume, claim)? {
                    return Ok(Some(volume.clone()));
                }
            }
        }

        Ok(None)
    }

    /// Check if a volume satisfies a claim's requirements
    fn volume_satisfies_claim(&self, volume: &PersistentVolume, claim: &PersistentVolumeClaim) -> Result<bool, StorageError> {
        // Check access modes
        let access_modes_match = claim.spec.access_modes.iter()
            .all(|mode| volume.spec.access_modes.contains(mode));

        // Check storage capacity
        let capacity_match = if let Some(requested) = claim.spec.resources.requests.get("storage") {
            let volume_capacity = self.volume_manager.parse_capacity(&volume.spec.capacity.storage)?;
            let requested_capacity = self.volume_manager.parse_capacity(requested)?;
            volume_capacity >= requested_capacity
        } else {
            true
        };

        // Check storage class
        let storage_class_match = match (&claim.spec.storage_class_name, &volume.spec.storage_class_name) {
            (Some(claim_class), Some(volume_class)) => claim_class == volume_class,
            (None, None) => true,
            _ => false,
        };

        Ok(access_modes_match && capacity_match && storage_class_match)
    }

    /// Bind a claim to a volume
    fn bind_claim_to_volume(&self, claim_id: &str, volume_id: &str) -> Result<(), StorageError> {
        // Update volume status
        {
            let mut volumes = self.volume_manager.volumes.lock().unwrap();
            if let Some(volume) = volumes.get_mut(volume_id) {
                volume.status.phase = VolumePhase::Bound;
                volume.status.message = Some(format!("Bound to claim {}", claim_id));
                volume.status.last_phase_transition_time = Utc::now();
                volume.updated_at = Utc::now();
            } else {
                return Err(StorageError::VolumeNotFound(volume_id.to_string()));
            }
        }

        // Update claim status
        {
            let mut claims = self.volume_manager.claims.lock().unwrap();
            if let Some(claim) = claims.get_mut(claim_id) {
                claim.status.phase = ClaimPhase::Bound;
                claim.status.conditions.push(PersistentVolumeClaimCondition {
                    condition_type: "Bound".to_string(),
                    status: "True".to_string(),
                    last_probe_time: Some(Utc::now()),
                    last_transition_time: Utc::now(),
                    reason: Some("VolumeBound".to_string()),
                    message: Some(format!("Bound to volume {}", volume_id)),
                });
                claim.updated_at = Utc::now();
            } else {
                return Err(StorageError::ClaimNotFound(claim_id.to_string()));
            }
        }

        Ok(())
    }

    /// Process volume recycling
    async fn process_recycling(&self) -> Result<(), StorageError> {
        let volume_ids: Vec<String> = {
            let mut queue = self.recycling_queue.lock().unwrap();
            queue.drain(..).collect()
        };

        for volume_id in volume_ids {
            if let Err(e) = self.recycle_volume(&volume_id).await {
                eprintln!("Failed to recycle volume {}: {}", volume_id, e);
            }
        }

        Ok(())
    }

    /// Recycle a released volume
    async fn recycle_volume(&self, volume_id: &str) -> Result<(), StorageError> {
        let volume = {
            let volumes = self.volume_manager.volumes.lock().unwrap();
            volumes.get(volume_id).cloned()
        };

        if let Some(volume) = volume {
            if matches!(volume.status.phase, VolumePhase::Released) {
                match volume.spec.persistent_volume_reclaim_policy {
                    ReclaimPolicy::Recycle => {
                        // Clear volume data and make available again
                        self.volume_manager.recycle_volume_storage(&volume.spec.volume_source)?;
                        
                        let mut volumes = self.volume_manager.volumes.lock().unwrap();
                        if let Some(vol) = volumes.get_mut(volume_id) {
                            vol.status.phase = VolumePhase::Available;
                            vol.status.message = Some("Volume recycled and available".to_string());
                            vol.status.last_phase_transition_time = Utc::now();
                            vol.updated_at = Utc::now();
                        }
                        
                        println!("Recycled volume {}", volume_id);
                    }
                    ReclaimPolicy::Delete => {
                        // Delete the volume
                        self.volume_manager.delete_persistent_volume(volume_id)?;
                        println!("Deleted released volume {}", volume_id);
                    }
                    ReclaimPolicy::Retain => {
                        // Do nothing, keep as released
                        println!("Retaining released volume {}", volume_id);
                    }
                }
            }
        }

        Ok(())
    }

    /// Cleanup orphaned resources
    async fn cleanup_orphaned_resources(&self) -> Result<(), StorageError> {
        // Find volumes that have been released for too long
        let now = Utc::now();
        let volumes_to_cleanup: Vec<String> = {
            let volumes = self.volume_manager.volumes.lock().unwrap();
            volumes.values()
                .filter(|volume| {
                    matches!(volume.status.phase, VolumePhase::Released) &&
                    now.signed_duration_since(volume.status.last_phase_transition_time)
                        .to_std()
                        .unwrap_or(Duration::ZERO) > Duration::from_secs(3600) // 1 hour
                })
                .map(|volume| volume.id.clone())
                .collect()
        };

        for volume_id in volumes_to_cleanup {
            self.enqueue_volume_for_recycling(volume_id);
        }

        Ok(())
    }

    /// Check if auto-scaling is needed
    async fn check_auto_scaling(&self) -> Result<(), StorageError> {
        let stats = self.volume_manager.get_storage_stats();
        
        let utilization = if stats.total_capacity > 0 {
            stats.allocated_capacity as f64 / stats.total_capacity as f64
        } else {
            0.0
        };

        if utilization > self.config.storage_efficiency_threshold {
            println!("Storage utilization ({:.2}%) exceeds threshold ({:.2}%), considering auto-scaling",
                    utilization * 100.0, self.config.storage_efficiency_threshold * 100.0);
            
            // In a real implementation, this would trigger creation of additional storage capacity
            self.trigger_storage_scaling().await?;
        }

        Ok(())
    }

    /// Trigger storage capacity scaling
    async fn trigger_storage_scaling(&self) -> Result<(), StorageError> {
        // Mock implementation - in production this would:
        // 1. Calculate required additional capacity
        // 2. Create new volumes based on demand patterns
        // 3. Coordinate with cloud provider APIs
        
        println!("Mock: Triggered storage auto-scaling");
        Ok(())
    }

    /// Get scheduler statistics
    pub fn get_scheduler_stats(&self) -> VolumeSchedulerStats {
        let provisioning_queue_size = self.provisioning_queue.lock().unwrap().len();
        let recycling_queue_size = self.recycling_queue.lock().unwrap().len();
        let storage_stats = self.volume_manager.get_storage_stats();

        VolumeSchedulerStats {
            pending_provisioning: provisioning_queue_size,
            pending_recycling: recycling_queue_size,
            storage_stats,
            dynamic_provisioning_enabled: self.config.dynamic_provisioning_enabled,
            auto_scaling_enabled: self.config.auto_scaling_enabled,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VolumeSchedulerStats {
    pub pending_provisioning: usize,
    pub pending_recycling: usize,
    pub storage_stats: crate::services::storage::StorageStats,
    pub dynamic_provisioning_enabled: bool,
    pub auto_scaling_enabled: bool,
}