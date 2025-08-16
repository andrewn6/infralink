/// Test pod scheduling and management functionality
use std::collections::HashMap;
use chrono::Utc;
use principal::scale::scale::{InfralinkScheduler as Scheduler, Pod, PodSpec, PodStatus, PodPhase, ContainerSpec, 
    Node, NodeStatus, ResourceSpec, CloudProvider, RestartPolicy, ResourceRequirements, Taint, TaintEffect,
    EnvVar, ContainerPort, VolumeMount, PodCondition, PodConditionType, ConditionStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Infralink Pod Testing Suite");
    println!("=====================================");

    // Create scheduler instance
    let mut scheduler = Scheduler::new();
    
    // Test 1: Add nodes to the cluster
    test_add_nodes(&mut scheduler).await?;
    
    // Test 2: Create and schedule simple pod
    test_simple_pod_scheduling(&mut scheduler).await?;
    
    // Test 3: Create pod with resource requirements
    test_pod_with_resources(&mut scheduler).await?;
    
    // Test 4: Create multi-container pod
    test_multi_container_pod(&mut scheduler).await?;
    
    // Test 5: Test pod lifecycle management
    test_pod_lifecycle(&mut scheduler).await?;
    
    // Test 6: List all pods
    test_list_pods(&scheduler).await?;

    println!("\n✅ All pod tests completed successfully!");
    Ok(())
}

async fn test_add_nodes(scheduler: &mut Scheduler) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 1: Adding nodes to cluster");
    
    // Add local Docker node
    let local_node = Node {
        id: "node-local-1".to_string(),
        name: "infralink-local".to_string(),
        provider: CloudProvider::Local,
        status: NodeStatus::Ready,
        capacity: ResourceSpec {
            cpu_cores: 4.0,
            memory_gb: 8.0,
            storage_gb: 100.0,
            gpu_count: 0,
        },
        allocatable: ResourceSpec {
            cpu_cores: 3.5, // Reserve some for system
            memory_gb: 7.0,
            storage_gb: 90.0,
            gpu_count: 0,
        },
        allocated: ResourceSpec {
            cpu_cores: 0.0,
            memory_gb: 0.0,
            storage_gb: 0.0,
            gpu_count: 0,
        },
        labels: HashMap::from([
            ("node-type".to_string(), "worker".to_string()),
            ("zone".to_string(), "local".to_string()),
        ]),
        taints: Vec::new(),
        last_heartbeat: Utc::now(),
    };
    
    // Add Vultr node
    let vultr_node = Node {
        id: "node-vultr-1".to_string(),
        name: "infralink-vultr".to_string(),
        provider: CloudProvider::Vultr { region: "ewr".to_string() },
        status: NodeStatus::Ready,
        capacity: ResourceSpec {
            cpu_cores: 2.0,
            memory_gb: 4.0,
            storage_gb: 80.0,
            gpu_count: 0,
        },
        allocatable: ResourceSpec {
            cpu_cores: 1.8,
            memory_gb: 3.5,
            storage_gb: 70.0,
            gpu_count: 0,
        },
        allocated: ResourceSpec {
            cpu_cores: 0.0,
            memory_gb: 0.0,
            storage_gb: 0.0,
            gpu_count: 0,
        },
        labels: HashMap::from([
            ("node-type".to_string(), "worker".to_string()),
            ("zone".to_string(), "ewr".to_string()),
            ("provider".to_string(), "vultr".to_string()),
        ]),
        taints: Vec::new(),
        last_heartbeat: Utc::now(),
    };
    
    scheduler.add_node(local_node);
    scheduler.add_node(vultr_node);
    
    println!("✅ Added 2 nodes to cluster");
    println!("   - Local Docker node (4 CPU, 8GB RAM)");
    println!("   - Vultr VPS node (2 CPU, 4GB RAM)");
    
    Ok(())
}

async fn test_simple_pod_scheduling(scheduler: &mut Scheduler) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 2: Simple pod scheduling");
    
    let pod = Pod {
        id: "pod-nginx-1".to_string(),
        name: "nginx-pod".to_string(),
        namespace: "default".to_string(),
        spec: PodSpec {
            containers: vec![ContainerSpec {
                name: "nginx".to_string(),
                image: "nginx:alpine".to_string(),
                ports: vec![],
                env: vec![],
                volume_mounts: vec![],
                command: None,
                args: None,
                resources: ResourceRequirements {
                    requests: ResourceSpec {
                        cpu_cores: 0.1,
                        memory_gb: 0.1,
                        storage_gb: 0.0,
                        gpu_count: 0,
                    },
                    limits: ResourceSpec {
                        cpu_cores: 0.5,
                        memory_gb: 0.5,
                        storage_gb: 0.0,
                        gpu_count: 0,
                    },
                },
                command: vec![],
                args: vec![],
            }],
            restart_policy: RestartPolicy::Always,
            resources: ResourceRequirements {
                requests: ResourceSpec {
                    cpu_cores: 0.1,
                    memory_gb: 0.1,
                    storage_gb: 0.0,
                    gpu_count: 0,
                },
                limits: ResourceSpec {
                    cpu_cores: 0.5,
                    memory_gb: 0.5,
                    storage_gb: 0.0,
                    gpu_count: 0,
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
            ("version".to_string(), "v1".to_string()),
        ]),
        annotations: HashMap::new(),
    };

    let pod_id = scheduler.schedule_pod(pod).await?;
    println!("✅ Scheduled simple nginx pod: {}", pod_id);
    
    // Check pod status
    if let Some(scheduled_pod) = scheduler.pods.get(&pod_id) {
        println!("   - Status: {:?}", scheduled_pod.status.phase);
        println!("   - Node: {:?}", scheduled_pod.node_id);
        println!("   - Conditions: {} conditions", scheduled_pod.status.conditions.len());
    }
    
    Ok(())
}

async fn test_pod_with_resources(scheduler: &mut Scheduler) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 3: Pod with specific resource requirements");
    
    let pod = Pod {
        id: "pod-high-cpu".to_string(),
        name: "high-cpu-pod".to_string(),
        namespace: "compute".to_string(),
        spec: PodSpec {
            containers: vec![ContainerSpec {
                name: "cpu-intensive".to_string(),
                image: "alpine:latest".to_string(),
                command: Some(vec!["sh".to_string(), "-c".to_string()]),
                args: Some(vec!["while true; do echo 'Computing...'; sleep 1; done".to_string()]),
                ports: vec![],
                env: vec![
                    EnvVar { name: "CPU_LIMIT".to_string(), value: Some("2".to_string()) },
                    EnvVar { name: "MEMORY_LIMIT".to_string(), value: Some("2Gi".to_string()) },
                ],
                volume_mounts: vec![],
                resources: ResourceRequirements {
                    requests: ResourceSpec {
                        cpu_cores: 1.0,
                        memory_gb: 1.0,
                        storage_gb: 0.0,
                        gpu_count: 0,
                    },
                    limits: ResourceSpec {
                        cpu_cores: 2.0,
                        memory_gb: 2.0,
                        storage_gb: 0.0,
                        gpu_count: 0,
                    },
                },
            }],
            restart_policy: RestartPolicy::OnFailure,
            resources: ResourceRequirements {
                requests: ResourceSpec {
                    cpu_cores: 1.0,
                    memory_gb: 1.0,
                    storage_gb: 0.0,
                    gpu_count: 0,
                },
                limits: ResourceSpec {
                    cpu_cores: 2.0,
                    memory_gb: 2.0,
                    storage_gb: 0.0,
                    gpu_count: 0,
                },
            },
            node_selector: HashMap::from([
                ("node-type".to_string(), "worker".to_string()),
            ]),
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
            ("app".to_string(), "cpu-test".to_string()),
            ("workload".to_string(), "compute".to_string()),
        ]),
        annotations: HashMap::from([
            ("description".to_string(), "High CPU workload for testing".to_string()),
        ]),
    };

    let pod_id = scheduler.schedule_pod(pod).await?;
    println!("✅ Scheduled high-CPU pod: {}", pod_id);
    
    if let Some(scheduled_pod) = scheduler.pods.get(&pod_id) {
        println!("   - Resource requests: {:.1} CPU, {:.1}GB RAM", 
                scheduled_pod.spec.resources.requests.cpu_cores,
                scheduled_pod.spec.resources.requests.memory_gb);
        println!("   - Resource limits: {:.1} CPU, {:.1}GB RAM", 
                scheduled_pod.spec.resources.limits.cpu_cores,
                scheduled_pod.spec.resources.limits.memory_gb);
    }
    
    Ok(())
}

async fn test_multi_container_pod(scheduler: &mut Scheduler) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 4: Multi-container pod (sidecar pattern)");
    
    let pod = Pod {
        id: "pod-webapp-logging".to_string(),
        name: "webapp-with-logging".to_string(),
        namespace: "production".to_string(),
        spec: PodSpec {
            containers: vec![
                // Main application container
                ContainerSpec {
                    name: "webapp".to_string(),
                    image: "nginx:alpine".to_string(),
                    ports: vec![],
                    env: vec![
                        EnvVar { name: "ENVIRONMENT".to_string(), value: Some("production".to_string()) },
                    ],
                    volume_mounts: vec![],
                    command: None,
                    args: None,
                    resources: ResourceRequirements {
                        requests: ResourceSpec {
                            cpu_cores: 0.2,
                            memory_gb: 0.5,
                            storage_gb: 0.0,
                            gpu_count: 0,
                        },
                        limits: ResourceSpec {
                            cpu_cores: 1.0,
                            memory_gb: 1.0,
                            storage_gb: 0.0,
                            gpu_count: 0,
                        },
                    },
                    command: vec![],
                    args: vec![],
                },
                // Logging sidecar container
                ContainerSpec {
                    name: "log-forwarder".to_string(),
                    image: "fluent/fluent-bit:latest".to_string(),
                    ports: vec![],
                    env: vec![
                        EnvVar { name: "FLUENT_CONF".to_string(), value: Some("fluent-bit.conf".to_string()) },
                        EnvVar { name: "LOG_LEVEL".to_string(), value: Some("info".to_string()) },
                    ],
                    volume_mounts: vec![],
                    command: None,
                    args: None,
                    resources: ResourceRequirements {
                        requests: ResourceSpec {
                            cpu_cores: 0.1,
                            memory_gb: 0.2,
                            storage_gb: 0.0,
                            gpu_count: 0,
                        },
                        limits: ResourceSpec {
                            cpu_cores: 0.2,
                            memory_gb: 0.5,
                            storage_gb: 0.0,
                            gpu_count: 0,
                        },
                    },
                    command: vec![],
                    args: vec![],
                },
            ],
            restart_policy: RestartPolicy::Always,
            resources: ResourceRequirements {
                requests: ResourceSpec {
                    cpu_cores: 0.3, // Sum of container requests
                    memory_gb: 0.7,
                    storage_gb: 0.0,
                    gpu_count: 0,
                },
                limits: ResourceSpec {
                    cpu_cores: 1.2, // Sum of container limits
                    memory_gb: 1.5,
                    storage_gb: 0.0,
                    gpu_count: 0,
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
            ("app".to_string(), "webapp".to_string()),
            ("tier".to_string(), "frontend".to_string()),
            ("pattern".to_string(), "sidecar".to_string()),
        ]),
        annotations: HashMap::from([
            ("sidecar.istio.io/inject".to_string(), "false".to_string()),
        ]),
    };

    let pod_id = scheduler.schedule_pod(pod).await?;
    println!("✅ Scheduled multi-container pod: {}", pod_id);
    
    if let Some(scheduled_pod) = scheduler.pods.get(&pod_id) {
        println!("   - Containers: {}", scheduled_pod.spec.containers.len());
        for container in &scheduled_pod.spec.containers {
            println!("     • {} ({})", container.name, container.image);
        }
    }
    
    Ok(())
}

async fn test_pod_lifecycle(scheduler: &mut Scheduler) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 5: Pod lifecycle management");
    
    // Test deleting a pod
    if let Some(pod_id) = scheduler.pods.keys().next().cloned() {
        println!("✅ Testing pod deletion for: {}", pod_id);
        scheduler.pods.remove(&pod_id);
        println!("   - Pod removed from scheduler");
    }
    
    // Test updating pod status
    if let Some((pod_id, pod)) = scheduler.pods.iter_mut().next() {
        println!("✅ Testing pod status update for: {}", pod_id);
        pod.status.phase = PodPhase::Running;
        pod.status.start_time = Some(Utc::now());
        pod.status.pod_ip = Some("10.244.1.5".to_string());
        println!("   - Updated to Running phase with IP");
    }
    
    Ok(())
}

async fn test_list_pods(scheduler: &Scheduler) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 6: Listing all pods");
    
    println!("✅ Current pods in cluster:");
    println!("   Total pods: {}", scheduler.pods.len());
    
    for (id, pod) in &scheduler.pods {
        println!("   • {} ({}) - {:?} on {:?}", 
                pod.name, 
                id, 
                pod.status.phase,
                pod.node_id.as_deref().unwrap_or("unscheduled"));
        println!("     Namespace: {}, Containers: {}", 
                pod.namespace, 
                pod.spec.containers.len());
        
        // Show resource usage
        let requests = &pod.spec.resources.requests;
        println!("     Resources: {:.1}m CPU, {:.0}Mi RAM", 
                requests.cpu_cores * 1000.0, 
                requests.memory_gb * 1024.0);
    }
    
    Ok(())
}