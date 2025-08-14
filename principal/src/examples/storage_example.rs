use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

use crate::services::storage::{
    PersistentVolumeManager, StorageProvider, LocalStorageProvider,
    PersistentVolumeSpec, PersistentVolumeClaimSpec, VolumeCapacity,
    AccessMode, ReclaimPolicy, VolumeSource, HostPathType,
    ResourceRequirements, VolumeMode,
};
use crate::services::volume_scheduler::{VolumeScheduler, VolumeSchedulerConfig};

/// Example demonstrating persistent volume management
pub async fn run_storage_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Infralink Persistent Volume Management Example ===");

    // Initialize storage provider (local for this example)
    let storage_provider = StorageProvider::Local(LocalStorageProvider {
        base_path: "/tmp/infralink/demo-volumes".to_string(),
        max_size: 50 * 1024 * 1024 * 1024, // 50GB
    });

    // Create volume manager
    let volume_manager = Arc::new(PersistentVolumeManager::new(storage_provider));

    // Create volume scheduler with custom config
    let scheduler_config = VolumeSchedulerConfig {
        dynamic_provisioning_enabled: true,
        auto_scaling_enabled: true,
        recycling_enabled: true,
        check_interval: Duration::from_secs(5), // Faster for demo
        max_pending_time: Duration::from_secs(30),
        storage_efficiency_threshold: 0.7,
    };

    let volume_scheduler = VolumeScheduler::new(volume_manager.clone(), scheduler_config);

    // Start the scheduler
    volume_scheduler.start().await;

    // Run examples
    demo_manual_volume_creation(&volume_manager).await?;
    demo_volume_claims(&volume_manager).await?;
    demo_dynamic_provisioning(&volume_manager, &volume_scheduler).await?;
    demo_storage_classes(&volume_manager).await?;
    demo_volume_lifecycle(&volume_manager).await?;

    // Show final statistics
    show_storage_statistics(&volume_manager, &volume_scheduler).await?;

    Ok(())
}

async fn demo_manual_volume_creation(
    volume_manager: &PersistentVolumeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n1. Manual Persistent Volume Creation");

    // Create a local volume
    let local_volume_spec = PersistentVolumeSpec {
        capacity: VolumeCapacity {
            storage: "10Gi".to_string(),
        },
        access_modes: vec![AccessMode::ReadWriteOnce],
        persistent_volume_reclaim_policy: ReclaimPolicy::Delete,
        storage_class_name: Some("local-storage".to_string()),
        mount_options: vec![],
        volume_source: VolumeSource::Local {
            path: "/tmp/infralink/demo-volumes/manual-volume-1".to_string(),
        },
        node_affinity: None,
    };

    let local_volume = volume_manager.create_persistent_volume(
        "demo-local-volume".to_string(),
        "default".to_string(),
        local_volume_spec,
    )?;

    println!("✓ Created local volume: {} ({})", local_volume.name, local_volume.id);

    // Create a host path volume
    let hostpath_volume_spec = PersistentVolumeSpec {
        capacity: VolumeCapacity {
            storage: "5Gi".to_string(),
        },
        access_modes: vec![AccessMode::ReadWriteMany],
        persistent_volume_reclaim_policy: ReclaimPolicy::Retain,
        storage_class_name: Some("local-storage".to_string()),
        mount_options: vec!["bind".to_string()],
        volume_source: VolumeSource::HostPath {
            path: "/tmp".to_string(),
            host_path_type: HostPathType::Directory,
        },
        node_affinity: None,
    };

    let hostpath_volume = volume_manager.create_persistent_volume(
        "demo-hostpath-volume".to_string(),
        "default".to_string(),
        hostpath_volume_spec,
    )?;

    println!("✓ Created host path volume: {} ({})", hostpath_volume.name, hostpath_volume.id);

    // Create a cloud volume (mock)
    let cloud_volume_spec = PersistentVolumeSpec {
        capacity: VolumeCapacity {
            storage: "100Gi".to_string(),
        },
        access_modes: vec![AccessMode::ReadWriteOnce],
        persistent_volume_reclaim_policy: ReclaimPolicy::Delete,
        storage_class_name: Some("fast-ssd".to_string()),
        mount_options: vec![],
        volume_source: VolumeSource::AwsEbs {
            volume_id: "vol-1234567890abcdef0".to_string(),
            fs_type: "ext4".to_string(),
        },
        node_affinity: None,
    };

    let cloud_volume = volume_manager.create_persistent_volume(
        "demo-cloud-volume".to_string(),
        "production".to_string(),
        cloud_volume_spec,
    )?;

    println!("✓ Created cloud volume: {} ({})", cloud_volume.name, cloud_volume.id);

    Ok(())
}

async fn demo_volume_claims(
    volume_manager: &PersistentVolumeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Persistent Volume Claims");

    // Create a claim that should bind to existing local volume
    let mut local_requests = HashMap::new();
    local_requests.insert("storage".to_string(), "8Gi".to_string());

    let local_claim_spec = PersistentVolumeClaimSpec {
        access_modes: vec![AccessMode::ReadWriteOnce],
        resources: ResourceRequirements {
            requests: local_requests,
            limits: HashMap::new(),
        },
        volume_name: None,
        storage_class_name: Some("local-storage".to_string()),
        volume_mode: Some(VolumeMode::Filesystem),
        selector: None,
    };

    let local_claim = volume_manager.create_persistent_volume_claim(
        "demo-local-claim".to_string(),
        "default".to_string(),
        local_claim_spec,
    )?;

    println!("✓ Created local claim: {} (status: {:?})", local_claim.name, local_claim.status.phase);

    // Create a claim for shared storage
    let mut shared_requests = HashMap::new();
    shared_requests.insert("storage".to_string(), "3Gi".to_string());

    let shared_claim_spec = PersistentVolumeClaimSpec {
        access_modes: vec![AccessMode::ReadWriteMany],
        resources: ResourceRequirements {
            requests: shared_requests,
            limits: HashMap::new(),
        },
        volume_name: None,
        storage_class_name: Some("local-storage".to_string()),
        volume_mode: Some(VolumeMode::Filesystem),
        selector: None,
    };

    let shared_claim = volume_manager.create_persistent_volume_claim(
        "demo-shared-claim".to_string(),
        "default".to_string(),
        shared_claim_spec,
    )?;

    println!("✓ Created shared claim: {} (status: {:?})", shared_claim.name, shared_claim.status.phase);

    // Create a claim for high-performance storage
    let mut fast_requests = HashMap::new();
    fast_requests.insert("storage".to_string(), "50Gi".to_string());

    let fast_claim_spec = PersistentVolumeClaimSpec {
        access_modes: vec![AccessMode::ReadWriteOnce],
        resources: ResourceRequirements {
            requests: fast_requests,
            limits: HashMap::new(),
        },
        volume_name: None,
        storage_class_name: Some("fast-ssd".to_string()),
        volume_mode: Some(VolumeMode::Filesystem),
        selector: None,
    };

    let fast_claim = volume_manager.create_persistent_volume_claim(
        "demo-fast-claim".to_string(),
        "production".to_string(),
        fast_claim_spec,
    )?;

    println!("✓ Created fast claim: {} (status: {:?})", fast_claim.name, fast_claim.status.phase);

    Ok(())
}

async fn demo_dynamic_provisioning(
    volume_manager: &PersistentVolumeManager,
    volume_scheduler: &VolumeScheduler,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Dynamic Provisioning");

    // Create a claim that requires dynamic provisioning
    let mut dynamic_requests = HashMap::new();
    dynamic_requests.insert("storage".to_string(), "20Gi".to_string());

    let dynamic_claim_spec = PersistentVolumeClaimSpec {
        access_modes: vec![AccessMode::ReadWriteOnce],
        resources: ResourceRequirements {
            requests: dynamic_requests,
            limits: HashMap::new(),
        },
        volume_name: None,
        storage_class_name: Some("local-storage".to_string()),
        volume_mode: Some(VolumeMode::Filesystem),
        selector: None,
    };

    let dynamic_claim = volume_manager.create_persistent_volume_claim(
        "demo-dynamic-claim".to_string(),
        "default".to_string(),
        dynamic_claim_spec,
    )?;

    println!("✓ Created claim requiring dynamic provisioning: {} (status: {:?})", 
             dynamic_claim.name, dynamic_claim.status.phase);

    // Trigger dynamic provisioning
    volume_scheduler.enqueue_claim_for_provisioning(dynamic_claim.id.clone());
    println!("✓ Enqueued claim for dynamic provisioning");

    // Wait a bit for the scheduler to process
    println!("⏳ Waiting for dynamic provisioning...");
    sleep(Duration::from_secs(6)).await;

    // Check if claim was bound
    let claims = volume_manager.list_persistent_volume_claims(Some("default"));
    if let Some(updated_claim) = claims.iter().find(|c| c.id == dynamic_claim.id) {
        println!("✓ Dynamic claim status: {:?}", updated_claim.status.phase);
    }

    Ok(())
}

async fn demo_storage_classes(
    volume_manager: &PersistentVolumeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Storage Classes");

    let storage_classes = volume_manager.storage_classes.lock().unwrap();
    println!("Available storage classes:");
    
    for (name, storage_class) in storage_classes.iter() {
        println!("  - {}: provisioner={}, reclaim_policy={:?}", 
                name, 
                storage_class.provisioner, 
                storage_class.reclaim_policy);
        
        if !storage_class.parameters.is_empty() {
            println!("    Parameters:");
            for (key, value) in &storage_class.parameters {
                println!("      {}: {}", key, value);
            }
        }
    }

    Ok(())
}

async fn demo_volume_lifecycle(
    volume_manager: &PersistentVolumeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n5. Volume Lifecycle Management");

    // Create a volume with recycle policy
    let recyclable_volume_spec = PersistentVolumeSpec {
        capacity: VolumeCapacity {
            storage: "1Gi".to_string(),
        },
        access_modes: vec![AccessMode::ReadWriteOnce],
        persistent_volume_reclaim_policy: ReclaimPolicy::Recycle,
        storage_class_name: Some("local-storage".to_string()),
        mount_options: vec![],
        volume_source: VolumeSource::Local {
            path: "/tmp/infralink/demo-volumes/recyclable".to_string(),
        },
        node_affinity: None,
    };

    let recyclable_volume = volume_manager.create_persistent_volume(
        "demo-recyclable-volume".to_string(),
        "default".to_string(),
        recyclable_volume_spec,
    )?;

    println!("✓ Created recyclable volume: {}", recyclable_volume.name);

    // Create and bind a claim
    let mut recycle_requests = HashMap::new();
    recycle_requests.insert("storage".to_string(), "1Gi".to_string());

    let recycle_claim_spec = PersistentVolumeClaimSpec {
        access_modes: vec![AccessMode::ReadWriteOnce],
        resources: ResourceRequirements {
            requests: recycle_requests,
            limits: HashMap::new(),
        },
        volume_name: Some(recyclable_volume.id.clone()),
        storage_class_name: Some("local-storage".to_string()),
        volume_mode: Some(VolumeMode::Filesystem),
        selector: None,
    };

    let recycle_claim = volume_manager.create_persistent_volume_claim(
        "demo-recycle-claim".to_string(),
        "default".to_string(),
        recycle_claim_spec,
    )?;

    println!("✓ Created and bound claim: {}", recycle_claim.name);

    // Delete the claim to release the volume
    volume_manager.delete_persistent_volume_claim(&recycle_claim.id)?;
    println!("✓ Deleted claim to release volume");

    // Check volume status
    let volumes = volume_manager.list_persistent_volumes();
    if let Some(volume) = volumes.iter().find(|v| v.id == recyclable_volume.id) {
        println!("✓ Volume status after claim deletion: {:?}", volume.status.phase);
    }

    Ok(())
}

async fn show_storage_statistics(
    volume_manager: &PersistentVolumeManager,
    volume_scheduler: &VolumeScheduler,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n6. Storage Statistics");

    let storage_stats = volume_manager.get_storage_stats();
    println!("Storage Overview:");
    println!("  Total volumes: {}", storage_stats.total_volumes);
    println!("  Total claims: {}", storage_stats.total_claims);
    println!("  Total capacity: {:.2} GB", storage_stats.total_capacity as f64 / (1024.0 * 1024.0 * 1024.0));
    println!("  Allocated capacity: {:.2} GB", storage_stats.allocated_capacity as f64 / (1024.0 * 1024.0 * 1024.0));
    println!("  Available capacity: {:.2} GB", storage_stats.available_capacity as f64 / (1024.0 * 1024.0 * 1024.0));

    println!("\nVolume Status Distribution:");
    for (phase, count) in &storage_stats.volume_count_by_phase {
        println!("  {}: {}", phase, count);
    }

    println!("\nClaim Status Distribution:");
    for (phase, count) in &storage_stats.claim_count_by_phase {
        println!("  {}: {}", phase, count);
    }

    let scheduler_stats = volume_scheduler.get_scheduler_stats();
    println!("\nScheduler Status:");
    println!("  Pending provisioning: {}", scheduler_stats.pending_provisioning);
    println!("  Pending recycling: {}", scheduler_stats.pending_recycling);
    println!("  Dynamic provisioning enabled: {}", scheduler_stats.dynamic_provisioning_enabled);
    println!("  Auto-scaling enabled: {}", scheduler_stats.auto_scaling_enabled);

    println!("\nAll Persistent Volumes:");
    let volumes = volume_manager.list_persistent_volumes();
    for volume in volumes {
        println!("  {} ({}): {} - {:?} - {}", 
                volume.name, 
                volume.namespace,
                volume.spec.capacity.storage,
                volume.status.phase,
                volume.spec.access_modes.len());
    }

    println!("\nAll Persistent Volume Claims:");
    let claims = volume_manager.list_persistent_volume_claims(None);
    for claim in claims {
        let requested_storage = claim.spec.resources.requests.get("storage").unwrap_or(&"unknown".to_string()).clone();
        println!("  {} ({}): {} - {:?}", 
                claim.name, 
                claim.namespace,
                requested_storage,
                claim.status.phase);
    }

    Ok(())
}