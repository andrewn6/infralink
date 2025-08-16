/// Simple pod testing demonstration
use std::collections::HashMap;
use chrono::Utc;
use principal::scale::scale::{InfralinkScheduler as Scheduler, Pod, PodSpec, PodStatus, PodPhase, ContainerSpec, 
    Node, NodeStatus, ResourceSpec, CloudProvider, RestartPolicy, ResourceRequirements, EnvVar};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Infralink Pod Testing Suite");
    println!("=====================================");

    // Create scheduler
    let mut scheduler = Scheduler::new();
    
    // Add a simple local node
    add_test_node(&mut scheduler);
    
    // Test basic pod scheduling
    test_simple_pod(&mut scheduler).await?;
    
    // Show results
    show_cluster_status(&scheduler);

    println!("\n✅ Pod tests completed successfully!");
    Ok(())
}

fn add_test_node(scheduler: &mut Scheduler) {
    println!("\n📋 Adding test node to cluster");
    
    let node = Node {
        id: "test-node-1".to_string(),
        name: "infralink-test-node".to_string(),
        provider: CloudProvider::Local,
        status: NodeStatus::Ready,
        capacity: ResourceSpec {
            cpu_cores: 4.0,
            memory_mb: 8192,    // 8GB in MB
            storage_gb: 100,
            network_mbps: 1000,
        },
        allocatable: ResourceSpec {
            cpu_cores: 3.5,
            memory_mb: 7168,    // 7GB in MB  
            storage_gb: 90,
            network_mbps: 1000,
        },
        allocated: ResourceSpec {
            cpu_cores: 0.0,
            memory_mb: 0,
            storage_gb: 0,
            network_mbps: 0,
        },
        labels: HashMap::from([
            ("node-type".to_string(), "worker".to_string()),
            ("zone".to_string(), "local".to_string()),
        ]),
        taints: Vec::new(),
        last_heartbeat: Utc::now(),
    };
    
    scheduler.add_node(node);
    println!("✅ Added local test node (4 CPU, 8GB RAM)");
}

async fn test_simple_pod(scheduler: &mut Scheduler) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Creating and scheduling simple pod");
    
    let pod = Pod {
        id: "test-pod-1".to_string(),
        name: "nginx-test".to_string(),
        namespace: "default".to_string(),
        spec: PodSpec {
            containers: vec![ContainerSpec {
                name: "nginx".to_string(),
                image: "nginx:alpine".to_string(),
                command: None,
                args: None,
                env: vec![
                    EnvVar { 
                        name: "ENVIRONMENT".to_string(), 
                        value: Some("test".to_string()),
                        value_from: None,
                    },
                ],
                ports: vec![],
                resources: ResourceRequirements {
                    requests: ResourceSpec {
                        cpu_cores: 0.1,
                        memory_mb: 128,     // 128MB
                        storage_gb: 0,
                        network_mbps: 0,
                    },
                    limits: ResourceSpec {
                        cpu_cores: 0.5,
                        memory_mb: 512,     // 512MB
                        storage_gb: 0,
                        network_mbps: 0,
                    },
                },
                volume_mounts: vec![],
            }],
            restart_policy: RestartPolicy::Always,
            resources: ResourceRequirements {
                requests: ResourceSpec {
                    cpu_cores: 0.1,
                    memory_mb: 128,
                    storage_gb: 0,
                    network_mbps: 0,
                },
                limits: ResourceSpec {
                    cpu_cores: 0.5,
                    memory_mb: 512,
                    storage_gb: 0,
                    network_mbps: 0,
                },
            },
            node_selector: HashMap::new(),
            tolerations: vec![],
            affinity: None,
        },
        status: PodStatus {
            phase: PodPhase::Pending,
            conditions: vec![],
            container_statuses: vec![],
            pod_ip: None,
            start_time: None,
        },
        node_id: None,
        created_at: Utc::now(),
        labels: HashMap::from([
            ("app".to_string(), "nginx".to_string()),
            ("env".to_string(), "test".to_string()),
        ]),
        annotations: HashMap::new(),
    };

    let pod_id = scheduler.schedule_pod(pod).await?;
    println!("✅ Successfully scheduled pod: {}", pod_id);
    
    // Check the scheduled pod
    if let Some(scheduled_pod) = scheduler.pods.get(&pod_id) {
        println!("   - Pod ID: {}", scheduled_pod.id);
        println!("   - Status: {:?}", scheduled_pod.status.phase);
        println!("   - Node: {:?}", scheduled_pod.node_id.as_deref().unwrap_or("unassigned"));
        println!("   - Containers: {}", scheduled_pod.spec.containers.len());
        println!("   - CPU Request: {:.1}m", scheduled_pod.spec.resources.requests.cpu_cores * 1000.0);
        println!("   - Memory Request: {}MB", scheduled_pod.spec.resources.requests.memory_mb);
    }
    
    Ok(())
}

fn show_cluster_status(scheduler: &Scheduler) {
    println!("\n📊 Cluster Status Summary");
    println!("========================");
    println!("Nodes: {}", scheduler.nodes.len());
    println!("Pods: {}", scheduler.pods.len());
    println!("Deployments: {}", scheduler.deployments.len());
    println!("Services: {}", scheduler.services.len());
    
    println!("\nNode Details:");
    for node in &scheduler.nodes {
        println!("  • {} ({:?})", node.name, node.status);
        println!("    Capacity: {:.1} CPU, {}MB RAM", 
                node.capacity.cpu_cores, node.capacity.memory_mb);
        println!("    Available: {:.1} CPU, {}MB RAM", 
                node.allocatable.cpu_cores, node.allocatable.memory_mb);
    }
    
    println!("\nPod Details:");
    for (id, pod) in &scheduler.pods {
        println!("  • {} ({}) - {:?}", 
                pod.name, 
                &id[0..8], 
                pod.status.phase);
        println!("    Node: {:?}", pod.node_id.as_deref().unwrap_or("unscheduled"));
        println!("    Namespace: {}", pod.namespace);
    }
}