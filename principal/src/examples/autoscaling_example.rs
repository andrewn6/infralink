use std::collections::HashMap;
use tokio::time::{sleep, Duration};

use crate::services::autoscaler::{
    AutoscalerManager, AutoscalerConfig, HpaSpec, VpaSpec, NodeGroup,
    ScaleTargetRef, MetricSpec, MetricType, ResourceMetricSpec, MetricTarget, MetricTargetType,
    VpaUpdatePolicy, VpaUpdateMode, NodeGroupStatus, TaintEffect,
};
use crate::services::metrics::{MetricsCollector, MetricsConfig};

/// Example demonstrating comprehensive cluster autoscaling
pub async fn run_autoscaling_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Infralink Cluster Autoscaling Example ===");

    // Initialize metrics collector
    let metrics_config = MetricsConfig {
        collection_interval: Duration::from_secs(5), // Fast for demo
        node_metrics_enabled: true,
        pod_metrics_enabled: true,
        custom_metrics_enabled: true,
        ..crate::services::metrics::MetricsConfig::default()
    };
    
    let metrics_collector = MetricsCollector::new(metrics_config).await?;
    metrics_collector.start().await;
    println!("✅ Started metrics collection system");

    // Initialize autoscaler
    let autoscaler_config = AutoscalerConfig {
        hpa_sync_interval: Duration::from_secs(10), // Fast for demo
        vpa_sync_interval: Duration::from_secs(15),
        cluster_sync_interval: Duration::from_secs(20),
        scale_up_threshold: 0.7,
        scale_down_threshold: 0.3,
        ..crate::services::autoscaler::AutoscalerConfig::default()
    };

    let autoscaler = AutoscalerManager::new(autoscaler_config);
    autoscaler.start().await;
    println!("✅ Started autoscaling controllers");

    // Give systems time to initialize
    sleep(Duration::from_secs(2)).await;

    // Demo horizontal pod autoscaling
    demo_horizontal_pod_autoscaling(&autoscaler).await?;
    
    // Demo vertical pod autoscaling
    demo_vertical_pod_autoscaling(&autoscaler).await?;
    
    // Demo cluster autoscaling
    demo_cluster_autoscaling(&autoscaler).await?;
    
    // Demo custom metrics scaling
    demo_custom_metrics_scaling(&autoscaler).await?;
    
    // Show final statistics
    show_autoscaling_statistics(&autoscaler, &metrics_collector).await?;

    println!("\n🎯 Autoscaling examples completed successfully!");
    Ok(())
}

async fn demo_horizontal_pod_autoscaling(autoscaler: &AutoscalerManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n1. Horizontal Pod Autoscaling (HPA)");

    // Create HPA for a web application
    let web_hpa = HpaSpec {
        id: String::new(), // Will be generated
        name: "web-app-hpa".to_string(),
        namespace: "production".to_string(),
        target_ref: ScaleTargetRef {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "web-app".to_string(),
        },
        min_replicas: 2,
        max_replicas: 10,
        metrics: vec![
            MetricSpec {
                metric_type: MetricType::Resource,
                resource: Some(ResourceMetricSpec {
                    name: "cpu".to_string(),
                    target: MetricTarget {
                        target_type: MetricTargetType::Utilization,
                        value: None,
                        average_value: None,
                        average_utilization: Some(70), // Scale up when CPU > 70%
                    },
                }),
                pods: None,
                object: None,
                external: None,
            },
            MetricSpec {
                metric_type: MetricType::Resource,
                resource: Some(ResourceMetricSpec {
                    name: "memory".to_string(),
                    target: MetricTarget {
                        target_type: MetricTargetType::Utilization,
                        value: None,
                        average_value: None,
                        average_utilization: Some(80), // Scale up when memory > 80%
                    },
                }),
                pods: None,
                object: None,
                external: None,
            },
        ],
        behavior: None, // Use default scaling behavior
        created_at: chrono::Utc::now(),
        status: Default::default(),
    };

    autoscaler.hpa_controller.create_hpa(web_hpa)?;
    println!("✅ Created HPA for web-app deployment");

    // Create HPA for API service
    let api_hpa = HpaSpec {
        id: String::new(),
        name: "api-service-hpa".to_string(),
        namespace: "production".to_string(),
        target_ref: ScaleTargetRef {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "api-service".to_string(),
        },
        min_replicas: 3,
        max_replicas: 20,
        metrics: vec![
            MetricSpec {
                metric_type: MetricType::Resource,
                resource: Some(ResourceMetricSpec {
                    name: "cpu".to_string(),
                    target: MetricTarget {
                        target_type: MetricTargetType::Utilization,
                        value: None,
                        average_value: None,
                        average_utilization: Some(60), // More aggressive scaling
                    },
                }),
                pods: None,
                object: None,
                external: None,
            },
        ],
        behavior: None,
        created_at: chrono::Utc::now(),
        status: Default::default(),
    };

    autoscaler.hpa_controller.create_hpa(api_hpa)?;
    println!("✅ Created HPA for api-service deployment");

    // Simulate workload changes
    println!("🔄 Simulating workload changes for HPA...");
    sleep(Duration::from_secs(15)).await; // Let HPA controller run a few cycles

    // Check HPA status
    let hpas = autoscaler.hpa_controller.hpa_specs.lock().unwrap();
    for hpa in hpas.values() {
        println!("   HPA {}: current={}, desired={} replicas", 
                 hpa.name, 
                 hpa.status.current_replicas, 
                 hpa.status.desired_replicas);
    }

    Ok(())
}

async fn demo_vertical_pod_autoscaling(autoscaler: &AutoscalerManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Vertical Pod Autoscaling (VPA)");

    // Create VPA for database
    let db_vpa = VpaSpec {
        id: String::new(),
        name: "database-vpa".to_string(),
        namespace: "production".to_string(),
        target_ref: ScaleTargetRef {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "database".to_string(),
        },
        update_policy: VpaUpdatePolicy {
            update_mode: VpaUpdateMode::Auto,
            min_replicas: Some(1),
        },
        resource_policy: None, // Use default policies
        created_at: chrono::Utc::now(),
        status: Default::default(),
    };

    autoscaler.vpa_controller.create_vpa(db_vpa)?;
    println!("✅ Created VPA for database deployment (Auto mode)");

    // Create VPA for cache service
    let cache_vpa = VpaSpec {
        id: String::new(),
        name: "cache-vpa".to_string(),
        namespace: "production".to_string(),
        target_ref: ScaleTargetRef {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "redis-cache".to_string(),
        },
        update_policy: VpaUpdatePolicy {
            update_mode: VpaUpdateMode::Recreation,
            min_replicas: Some(1),
        },
        resource_policy: None,
        created_at: chrono::Utc::now(),
        status: Default::default(),
    };

    autoscaler.vpa_controller.create_vpa(cache_vpa)?;
    println!("✅ Created VPA for redis-cache deployment (Recreation mode)");

    // Simulate resource usage analysis
    println!("📊 Analyzing resource usage patterns for VPA...");
    sleep(Duration::from_secs(20)).await; // Let VPA controller analyze

    // Check VPA recommendations
    let vpas = autoscaler.vpa_controller.vpa_specs.lock().unwrap();
    for vpa in vpas.values() {
        println!("   VPA {}: mode={:?}", vpa.name, vpa.update_policy.update_mode);
        if let Some(recommendation) = &vpa.status.recommendation {
            for container_rec in &recommendation.container_recommendations {
                println!("     Container {}: target CPU={}, memory={}", 
                         container_rec.container_name,
                         container_rec.target.get("cpu").unwrap_or(&"unknown".to_string()),
                         container_rec.target.get("memory").unwrap_or(&"unknown".to_string()));
            }
        }
    }

    Ok(())
}

async fn demo_cluster_autoscaling(autoscaler: &AutoscalerManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Cluster Autoscaling");

    // Create node groups for different workload types
    let general_node_group = NodeGroup {
        id: String::new(),
        name: "general-purpose".to_string(),
        min_size: 1,
        max_size: 5,
        desired_capacity: 2,
        instance_type: "t3.medium".to_string(),
        availability_zones: vec!["us-west-2a".to_string(), "us-west-2b".to_string()],
        labels: {
            let mut labels = HashMap::new();
            labels.insert("workload-type".to_string(), "general".to_string());
            labels.insert("node-group".to_string(), "general-purpose".to_string());
            labels
        },
        taints: vec![],
        auto_scaling_enabled: true,
        created_at: chrono::Utc::now(),
        status: NodeGroupStatus {
            ready_nodes: 2,
            total_nodes: 2,
            conditions: vec![],
            last_scale_time: None,
        },
    };

    autoscaler.cluster_autoscaler.create_node_group(general_node_group)?;
    println!("✅ Created general-purpose node group (1-5 nodes)");

    // Create compute-optimized node group
    let compute_node_group = NodeGroup {
        id: String::new(),
        name: "compute-optimized".to_string(),
        min_size: 0,
        max_size: 10,
        desired_capacity: 0,
        instance_type: "c5.large".to_string(),
        availability_zones: vec!["us-west-2a".to_string()],
        labels: {
            let mut labels = HashMap::new();
            labels.insert("workload-type".to_string(), "compute".to_string());
            labels.insert("node-group".to_string(), "compute-optimized".to_string());
            labels
        },
        taints: vec![
            crate::services::autoscaler::NodeTaint {
                key: "workload".to_string(),
                value: Some("compute".to_string()),
                effect: TaintEffect::NoSchedule,
            },
        ],
        auto_scaling_enabled: true,
        created_at: chrono::Utc::now(),
        status: NodeGroupStatus {
            ready_nodes: 0,
            total_nodes: 0,
            conditions: vec![],
            last_scale_time: None,
        },
    };

    autoscaler.cluster_autoscaler.create_node_group(compute_node_group)?;
    println!("✅ Created compute-optimized node group (0-10 nodes)");

    // Simulate unschedulable pods requiring scale up
    println!("🚀 Simulating unschedulable pods requiring cluster scale up...");
    for i in 1..=3 {
        let pod_id = format!("unschedulable-pod-{}", i);
        autoscaler.cluster_autoscaler.report_unschedulable_pod(pod_id);
    }

    sleep(Duration::from_secs(25)).await; // Let cluster autoscaler run

    // Check cluster scaling activities
    let activities = autoscaler.cluster_autoscaler.scaling_activities.lock().unwrap();
    println!("   Cluster scaling activities: {}", activities.len());
    for activity in activities.iter().take(3) {
        println!("     {:?} {}: {} - {:?}", 
                 activity.activity_type, 
                 activity.node_group_name,
                 activity.description,
                 activity.status_code);
    }

    // Show node group status
    let node_groups = autoscaler.cluster_autoscaler.node_groups.lock().unwrap();
    for node_group in node_groups.values() {
        println!("   Node group {}: {}/{} nodes ready", 
                 node_group.name,
                 node_group.status.ready_nodes,
                 node_group.status.total_nodes);
    }

    Ok(())
}

async fn demo_custom_metrics_scaling(autoscaler: &AutoscalerManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Custom Metrics Scaling");

    // Register custom application metrics
    let queue_length_metric = crate::services::autoscaler::CustomMetric {
        name: "queue_length".to_string(),
        namespace: "production".to_string(),
        value: 45.0,
        unit: "messages".to_string(),
        labels: {
            let mut labels = HashMap::new();
            labels.insert("queue_name".to_string(), "email_queue".to_string());
            labels.insert("service".to_string(), "email-processor".to_string());
            labels
        },
        timestamp: chrono::Utc::now(),
        ttl: Duration::from_secs(300),
    };

    autoscaler.register_custom_metric(queue_length_metric);
    println!("✅ Registered queue_length custom metric");

    let response_time_metric = crate::services::autoscaler::CustomMetric {
        name: "avg_response_time".to_string(),
        namespace: "production".to_string(),
        value: 850.0,
        unit: "milliseconds".to_string(),
        labels: {
            let mut labels = HashMap::new();
            labels.insert("service".to_string(), "api-gateway".to_string());
            labels.insert("endpoint".to_string(), "/api/v1".to_string());
            labels
        },
        timestamp: chrono::Utc::now(),
        ttl: Duration::from_secs(300),
    };

    autoscaler.register_custom_metric(response_time_metric);
    println!("✅ Registered avg_response_time custom metric");

    let error_rate_metric = crate::services::autoscaler::CustomMetric {
        name: "error_rate".to_string(),
        namespace: "production".to_string(),
        value: 2.3,
        unit: "percent".to_string(),
        labels: {
            let mut labels = HashMap::new();
            labels.insert("service".to_string(), "payment-service".to_string());
            labels
        },
        timestamp: chrono::Utc::now(),
        ttl: Duration::from_secs(300),
    };

    autoscaler.register_custom_metric(error_rate_metric);
    println!("✅ Registered error_rate custom metric");

    // Create HPA that uses external metrics
    let external_hpa = HpaSpec {
        id: String::new(),
        name: "queue-processor-hpa".to_string(),
        namespace: "production".to_string(),
        target_ref: ScaleTargetRef {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "queue-processor".to_string(),
        },
        min_replicas: 1,
        max_replicas: 15,
        metrics: vec![
            MetricSpec {
                metric_type: MetricType::External,
                resource: None,
                pods: None,
                object: None,
                external: Some(crate::services::autoscaler::ExternalMetricSpec {
                    metric: crate::services::autoscaler::MetricIdentifier {
                        name: "queue_length".to_string(),
                        selector: Some({
                            let mut selector = HashMap::new();
                            selector.insert("queue_name".to_string(), "email_queue".to_string());
                            selector
                        }),
                    },
                    target: MetricTarget {
                        target_type: MetricTargetType::Value,
                        value: Some("30".to_string()), // Scale up when queue > 30 messages
                        average_value: None,
                        average_utilization: None,
                    },
                }),
            },
        ],
        behavior: None,
        created_at: chrono::Utc::now(),
        status: Default::default(),
    };

    autoscaler.hpa_controller.create_hpa(external_hpa)?;
    println!("✅ Created HPA using custom queue_length metric");

    // Simulate metric changes over time
    println!("📈 Simulating custom metric changes...");
    for i in 1..=5 {
        sleep(Duration::from_secs(3)).await;
        
        let new_queue_metric = crate::services::autoscaler::CustomMetric {
            name: "queue_length".to_string(),
            namespace: "production".to_string(),
            value: 45.0 + (i as f64 * 10.0), // Increasing queue length
            unit: "messages".to_string(),
            labels: {
                let mut labels = HashMap::new();
                labels.insert("queue_name".to_string(), "email_queue".to_string());
                labels.insert("service".to_string(), "email-processor".to_string());
                labels
            },
            timestamp: chrono::Utc::now(),
            ttl: Duration::from_secs(300),
        };

        autoscaler.register_custom_metric(new_queue_metric);
        println!("   Updated queue_length to {}", 45.0 + (i as f64 * 10.0));
    }

    Ok(())
}

async fn show_autoscaling_statistics(
    autoscaler: &AutoscalerManager,
    metrics_collector: &MetricsCollector,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n5. Autoscaling Statistics Summary");

    let autoscaling_stats = autoscaler.get_autoscaling_stats();
    println!("📊 Autoscaling Overview:");
    println!("   HPAs: {}", autoscaling_stats.hpa_count);
    println!("   VPAs: {}", autoscaling_stats.vpa_count);
    println!("   Node Groups: {}", autoscaling_stats.node_groups_count);
    println!("   Total Nodes: {}", autoscaling_stats.total_nodes);
    println!("   Ready Nodes: {}", autoscaling_stats.ready_nodes);
    println!("   Scaling Events: {}", autoscaling_stats.scaling_events_count);
    println!("   Custom Metrics: {}", autoscaling_stats.custom_metrics_count);

    let cluster_metrics = metrics_collector.get_cluster_metrics();
    println!("\n📈 Cluster Resource Utilization:");
    println!("   CPU Usage: {:.2} / {:.2} cores ({:.1}%)", 
             cluster_metrics.cluster_resource_usage.cpu_usage_cores,
             cluster_metrics.cluster_resource_capacity.cpu_capacity_cores,
             if cluster_metrics.cluster_resource_capacity.cpu_capacity_cores > 0.0 {
                 (cluster_metrics.cluster_resource_usage.cpu_usage_cores / 
                  cluster_metrics.cluster_resource_capacity.cpu_capacity_cores) * 100.0
             } else { 0.0 });
    
    println!("   Memory Usage: {:.1} / {:.1} GB ({:.1}%)", 
             cluster_metrics.cluster_resource_usage.memory_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
             cluster_metrics.cluster_resource_capacity.memory_capacity_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
             if cluster_metrics.cluster_resource_capacity.memory_capacity_bytes > 0 {
                 (cluster_metrics.cluster_resource_usage.memory_usage_bytes as f64 / 
                  cluster_metrics.cluster_resource_capacity.memory_capacity_bytes as f64) * 100.0
             } else { 0.0 });

    println!("   Pod Usage: {} / {} pods ({:.1}%)", 
             cluster_metrics.total_pods,
             cluster_metrics.cluster_resource_capacity.pod_capacity,
             if cluster_metrics.cluster_resource_capacity.pod_capacity > 0 {
                 (cluster_metrics.total_pods as f64 / 
                  cluster_metrics.cluster_resource_capacity.pod_capacity as f64) * 100.0
             } else { 0.0 });

    // Show recent scaling events
    let scaling_history = autoscaler.hpa_controller.scaling_history.lock().unwrap();
    if !scaling_history.is_empty() {
        println!("\n🎯 Recent Scaling Events:");
        for event in scaling_history.iter().rev().take(5) {
            println!("   {} {:?} {}: {} → {} replicas ({})", 
                     event.timestamp.format("%H:%M:%S"),
                     event.event_type,
                     event.target_ref.name,
                     event.old_replicas,
                     event.new_replicas,
                     event.reason);
        }
    }

    // Show node group details
    let node_groups = autoscaler.cluster_autoscaler.node_groups.lock().unwrap();
    if !node_groups.is_empty() {
        println!("\n🖥️  Node Group Details:");
        for node_group in node_groups.values() {
            println!("   {} ({}): {}/{}/{} nodes (ready/total/max)", 
                     node_group.name,
                     node_group.instance_type,
                     node_group.status.ready_nodes,
                     node_group.status.total_nodes,
                     node_group.max_size);
        }
    }

    // Show custom metrics
    let custom_metrics = autoscaler.custom_metrics.lock().unwrap();
    if !custom_metrics.is_empty() {
        println!("\n📊 Custom Metrics:");
        for (name, metric) in custom_metrics.iter() {
            println!("   {}: {:.1} {} ({})", 
                     name, 
                     metric.value, 
                     metric.unit,
                     metric.timestamp.format("%H:%M:%S"));
        }
    }

    Ok(())
}

use crate::services::autoscaler::{HpaStatus, VpaStatus};

trait Default {
    fn default() -> Self;
}

impl Default for HpaStatus {
    fn default() -> Self {
        Self {
            current_replicas: 0,
            desired_replicas: 0,
            current_metrics: vec![],
            conditions: vec![],
            last_scale_time: None,
        }
    }
}

impl Default for VpaStatus {
    fn default() -> Self {
        Self {
            conditions: vec![],
            recommendation: None,
        }
    }
}