use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::providers::local::docker_client::{DockerClient, ContainerStats};

/// Metrics collection system for autoscaling and monitoring
#[derive(Clone)]
pub struct MetricsCollector {
    pub pod_metrics: Arc<Mutex<HashMap<String, PodMetrics>>>,
    pub node_metrics: Arc<Mutex<HashMap<String, NodeMetrics>>>,
    pub cluster_metrics: Arc<Mutex<ClusterMetrics>>,
    pub docker_client: Option<Arc<DockerClient>>,
    pub config: MetricsConfig,
}

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub collection_interval: Duration,
    pub metrics_retention_period: Duration,
    pub node_metrics_enabled: bool,
    pub pod_metrics_enabled: bool,
    pub custom_metrics_enabled: bool,
    pub scrape_timeout: Duration,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(15),
            metrics_retention_period: Duration::from_secs(3600), // 1 hour
            node_metrics_enabled: true,
            pod_metrics_enabled: true,
            custom_metrics_enabled: true,
            scrape_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodMetrics {
    pub pod_id: String,
    pub namespace: String,
    pub node_name: String,
    pub containers: HashMap<String, ContainerMetrics>,
    pub timestamp: DateTime<Utc>,
    pub window: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub name: String,
    pub usage: ResourceUsage,
    pub limits: ResourceLimits,
    pub requests: ResourceRequests,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu: CpuUsage,
    pub memory: MemoryUsage,
    pub network: NetworkUsage,
    pub storage: StorageUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub usage_nanocores: u64,
    pub usage_core_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub usage_bytes: u64,
    pub working_set_bytes: u64,
    pub rss_bytes: u64,
    pub page_faults: u64,
    pub major_page_faults: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    pub rx_bytes: u64,
    pub rx_errors: u64,
    pub tx_bytes: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub used_bytes: u64,
    pub capacity_bytes: u64,
    pub available_bytes: u64,
    pub inodes_used: u64,
    pub inodes_free: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub ephemeral_storage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequests {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub ephemeral_storage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_name: String,
    pub usage: NodeResourceUsage,
    pub capacity: NodeResourceCapacity,
    pub conditions: Vec<NodeCondition>,
    pub timestamp: DateTime<Utc>,
    pub window: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResourceUsage {
    pub cpu: CpuUsage,
    pub memory: MemoryUsage,
    pub network: NetworkUsage,
    pub storage: StorageUsage,
    pub pod_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResourceCapacity {
    pub cpu: String,
    pub memory: String,
    pub ephemeral_storage: String,
    pub pods: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCondition {
    pub condition_type: String,
    pub status: String,
    pub last_heartbeat_time: DateTime<Utc>,
    pub last_transition_time: DateTime<Utc>,
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub total_nodes: u32,
    pub ready_nodes: u32,
    pub total_pods: u32,
    pub running_pods: u32,
    pub pending_pods: u32,
    pub failed_pods: u32,
    pub cluster_resource_usage: ClusterResourceUsage,
    pub cluster_resource_capacity: ClusterResourceCapacity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResourceUsage {
    pub cpu_usage_cores: f64,
    pub memory_usage_bytes: u64,
    pub storage_usage_bytes: u64,
    pub network_usage_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResourceCapacity {
    pub cpu_capacity_cores: f64,
    pub memory_capacity_bytes: u64,
    pub storage_capacity_bytes: u64,
    pub pod_capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSample {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

impl MetricsCollector {
    pub async fn new(config: MetricsConfig) -> Result<Self, MetricsError> {
        let docker_client = if config.pod_metrics_enabled {
            Some(Arc::new(DockerClient::new().await
                .map_err(|e| MetricsError::InitializationFailed(e.to_string()))?))
        } else {
            None
        };

        Ok(Self {
            pod_metrics: Arc::new(Mutex::new(HashMap::new())),
            node_metrics: Arc::new(Mutex::new(HashMap::new())),
            cluster_metrics: Arc::new(Mutex::new(ClusterMetrics::default())),
            docker_client,
            config,
        })
    }

    /// Start metrics collection
    pub async fn start(&self) {
        println!("Starting metrics collector with config: {:?}", self.config);

        // Start pod metrics collection
        if self.config.pod_metrics_enabled {
            let collector = self.clone();
            tokio::spawn(async move {
                collector.collect_pod_metrics_loop().await;
            });
        }

        // Start node metrics collection
        if self.config.node_metrics_enabled {
            let collector = self.clone();
            tokio::spawn(async move {
                collector.collect_node_metrics_loop().await;
            });
        }

        // Start cluster metrics aggregation
        let collector = self.clone();
        tokio::spawn(async move {
            collector.aggregate_cluster_metrics_loop().await;
        });

        // Start metrics cleanup
        let collector = self.clone();
        tokio::spawn(async move {
            collector.cleanup_expired_metrics_loop().await;
        });

        println!("Metrics collection started");
    }

    /// Get pod metrics for a specific pod
    pub fn get_pod_metrics(&self, pod_id: &str) -> Option<PodMetrics> {
        let metrics = self.pod_metrics.lock().unwrap();
        metrics.get(pod_id).cloned()
    }

    /// Get node metrics for a specific node
    pub fn get_node_metrics(&self, node_name: &str) -> Option<NodeMetrics> {
        let metrics = self.node_metrics.lock().unwrap();
        metrics.get(node_name).cloned()
    }

    /// Get current cluster metrics
    pub fn get_cluster_metrics(&self) -> ClusterMetrics {
        let metrics = self.cluster_metrics.lock().unwrap();
        metrics.clone()
    }

    /// Get resource utilization for a pod (for HPA)
    pub fn get_pod_resource_utilization(&self, pod_id: &str, resource_name: &str) -> Option<f64> {
        let metrics = self.pod_metrics.lock().unwrap();
        if let Some(pod_metrics) = metrics.get(pod_id) {
            for container_metrics in pod_metrics.containers.values() {
                match resource_name {
                    "cpu" => {
                        // Convert nanocores to millicores, then to percentage
                        let millicores = container_metrics.usage.cpu.usage_nanocores as f64 / 1_000_000.0;
                        let request_millicores = self.parse_cpu_request(&container_metrics.requests.cpu);
                        if request_millicores > 0.0 {
                            return Some((millicores / request_millicores) * 100.0);
                        }
                    }
                    "memory" => {
                        let usage_bytes = container_metrics.usage.memory.usage_bytes as f64;
                        let request_bytes = self.parse_memory_request(&container_metrics.requests.memory);
                        if request_bytes > 0.0 {
                            return Some((usage_bytes / request_bytes) * 100.0);
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// Collect pod metrics loop
    async fn collect_pod_metrics_loop(&self) {
        let mut interval = interval(self.config.collection_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.collect_pod_metrics().await {
                eprintln!("Pod metrics collection error: {}", e);
            }
        }
    }

    /// Collect metrics for all pods
    async fn collect_pod_metrics(&self) -> Result<(), MetricsError> {
        if let Some(docker_client) = &self.docker_client {
            let containers = docker_client.list_containers(false).await
                .map_err(|e| MetricsError::CollectionFailed(e.to_string()))?;

            let mut pod_metrics_map = HashMap::new();

            for container in containers {
                if container.labels.get("infralink.managed").map_or(false, |v| v == "true") {
                    let pod_id = container.labels.get("infralink.pod.id")
                        .unwrap_or(&container.id).clone();

                    // Get container stats
                    match docker_client.get_container_stats(&container.id).await {
                        Ok(stats) => {
                            let container_metrics = self.convert_docker_stats_to_metrics(&container.name, &stats);
                            
                            let pod_metrics = pod_metrics_map.entry(pod_id.clone()).or_insert_with(|| {
                                PodMetrics {
                                    pod_id: pod_id.clone(),
                                    namespace: container.labels.get("infralink.namespace")
                                        .unwrap_or(&"default".to_string()).clone(),
                                    node_name: "local-node".to_string(),
                                    containers: HashMap::new(),
                                    timestamp: Utc::now(),
                                    window: self.config.collection_interval,
                                }
                            });

                            pod_metrics.containers.insert(container.name.clone(), container_metrics);
                        }
                        Err(e) => {
                            eprintln!("Failed to get stats for container {}: {}", container.id, e);
                        }
                    }
                }
            }

            // Update pod metrics
            let mut metrics = self.pod_metrics.lock().unwrap();
            for (pod_id, pod_metric) in pod_metrics_map {
                metrics.insert(pod_id, pod_metric);
            }
        }

        Ok(())
    }

    /// Collect node metrics loop
    async fn collect_node_metrics_loop(&self) {
        let mut interval = interval(self.config.collection_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.collect_node_metrics().await {
                eprintln!("Node metrics collection error: {}", e);
            }
        }
    }

    /// Collect metrics for all nodes
    async fn collect_node_metrics(&self) -> Result<(), MetricsError> {
        // Mock node metrics collection - in reality this would collect from kubelet or node exporter
        let node_metrics = NodeMetrics {
            node_name: "local-node".to_string(),
            usage: NodeResourceUsage {
                cpu: CpuUsage {
                    usage_nanocores: (rand::random::<f64>() * 2_000_000_000.0) as u64, // 0-2 cores
                    usage_core_seconds: 0.0,
                },
                memory: MemoryUsage {
                    usage_bytes: ((4.0 + rand::random::<f64>() * 4.0) * 1024.0 * 1024.0 * 1024.0) as u64, // 4-8GB
                    working_set_bytes: 0,
                    rss_bytes: 0,
                    page_faults: 0,
                    major_page_faults: 0,
                },
                network: NetworkUsage {
                    rx_bytes: (rand::random::<f64>() * 1_000_000.0) as u64,
                    rx_errors: 0,
                    tx_bytes: (rand::random::<f64>() * 1_000_000.0) as u64,
                    tx_errors: 0,
                },
                storage: StorageUsage {
                    used_bytes: ((50.0 + rand::random::<f64>() * 50.0) * 1024.0 * 1024.0 * 1024.0) as u64, // 50-100GB
                    capacity_bytes: 500 * 1024 * 1024 * 1024, // 500GB
                    available_bytes: 0,
                    inodes_used: 0,
                    inodes_free: 0,
                },
                pod_count: self.count_running_pods() as u32,
            },
            capacity: NodeResourceCapacity {
                cpu: "4".to_string(),
                memory: "16Gi".to_string(),
                ephemeral_storage: "500Gi".to_string(),
                pods: "110".to_string(),
            },
            conditions: vec![
                NodeCondition {
                    condition_type: "Ready".to_string(),
                    status: "True".to_string(),
                    last_heartbeat_time: Utc::now(),
                    last_transition_time: Utc::now(),
                    reason: "KubeletReady".to_string(),
                    message: "kubelet is posting ready status".to_string(),
                },
            ],
            timestamp: Utc::now(),
            window: self.config.collection_interval,
        };

        let mut metrics = self.node_metrics.lock().unwrap();
        metrics.insert("local-node".to_string(), node_metrics);

        Ok(())
    }

    /// Aggregate cluster metrics loop
    async fn aggregate_cluster_metrics_loop(&self) {
        let mut interval = interval(self.config.collection_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.aggregate_cluster_metrics().await {
                eprintln!("Cluster metrics aggregation error: {}", e);
            }
        }
    }

    /// Aggregate cluster-wide metrics
    async fn aggregate_cluster_metrics(&self) -> Result<(), MetricsError> {
        let node_metrics = self.node_metrics.lock().unwrap().clone();
        let pod_metrics = self.pod_metrics.lock().unwrap().clone();

        let mut cluster_cpu_usage = 0.0;
        let mut cluster_memory_usage = 0u64;
        let mut cluster_cpu_capacity = 0.0;
        let mut cluster_memory_capacity = 0u64;

        let ready_nodes = node_metrics.values()
            .filter(|node| node.conditions.iter()
                .any(|cond| cond.condition_type == "Ready" && cond.status == "True"))
            .count() as u32;

        for node in node_metrics.values() {
            cluster_cpu_usage += node.usage.cpu.usage_nanocores as f64 / 1_000_000_000.0; // Convert to cores
            cluster_memory_usage += node.usage.memory.usage_bytes;
            cluster_cpu_capacity += self.parse_cpu_capacity(&node.capacity.cpu);
            cluster_memory_capacity += self.parse_memory_capacity(&node.capacity.memory);
        }

        let total_pods = pod_metrics.len() as u32;
        let running_pods = pod_metrics.values()
            .filter(|pod| !pod.containers.is_empty())
            .count() as u32;

        let cluster_metrics = ClusterMetrics {
            total_nodes: node_metrics.len() as u32,
            ready_nodes,
            total_pods,
            running_pods,
            pending_pods: 0, // Would need to check pod status
            failed_pods: 0,  // Would need to check pod status
            cluster_resource_usage: ClusterResourceUsage {
                cpu_usage_cores: cluster_cpu_usage,
                memory_usage_bytes: cluster_memory_usage,
                storage_usage_bytes: 0, // Would aggregate storage usage
                network_usage_bytes_per_sec: 0, // Would calculate from deltas
            },
            cluster_resource_capacity: ClusterResourceCapacity {
                cpu_capacity_cores: cluster_cpu_capacity,
                memory_capacity_bytes: cluster_memory_capacity,
                storage_capacity_bytes: 0, // Would aggregate storage capacity
                pod_capacity: ready_nodes * 110, // Assuming 110 pods per node
            },
            timestamp: Utc::now(),
        };

        let mut metrics = self.cluster_metrics.lock().unwrap();
        *metrics = cluster_metrics;

        Ok(())
    }

    /// Cleanup expired metrics loop
    async fn cleanup_expired_metrics_loop(&self) {
        let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes

        loop {
            interval.tick().await;

            self.cleanup_expired_metrics().await;
        }
    }

    /// Remove metrics older than retention period
    async fn cleanup_expired_metrics(&self) {
        let cutoff_time = Utc::now() - chrono::Duration::from_std(self.config.metrics_retention_period).unwrap();

        // Clean pod metrics
        {
            let mut pod_metrics = self.pod_metrics.lock().unwrap();
            pod_metrics.retain(|_, metrics| metrics.timestamp > cutoff_time);
        }

        // Clean node metrics
        {
            let mut node_metrics = self.node_metrics.lock().unwrap();
            node_metrics.retain(|_, metrics| metrics.timestamp > cutoff_time);
        }
    }

    /// Convert Docker stats to container metrics
    fn convert_docker_stats_to_metrics(&self, container_name: &str, stats: &ContainerStats) -> ContainerMetrics {
        ContainerMetrics {
            name: container_name.to_string(),
            usage: ResourceUsage {
                cpu: CpuUsage {
                    usage_nanocores: (stats.cpu_usage_percent * 10_000_000.0) as u64, // Rough conversion
                    usage_core_seconds: 0.0,
                },
                memory: MemoryUsage {
                    usage_bytes: stats.memory_usage_bytes,
                    working_set_bytes: stats.memory_usage_bytes,
                    rss_bytes: 0,
                    page_faults: 0,
                    major_page_faults: 0,
                },
                network: NetworkUsage {
                    rx_bytes: stats.network_rx_bytes,
                    rx_errors: 0,
                    tx_bytes: stats.network_tx_bytes,
                    tx_errors: 0,
                },
                storage: StorageUsage {
                    used_bytes: stats.block_read_bytes + stats.block_write_bytes,
                    capacity_bytes: 0,
                    available_bytes: 0,
                    inodes_used: 0,
                    inodes_free: 0,
                },
            },
            limits: ResourceLimits {
                cpu: None,
                memory: Some(format!("{}bytes", stats.memory_limit_bytes)),
                ephemeral_storage: None,
            },
            requests: ResourceRequests {
                cpu: Some("100m".to_string()), // Default request
                memory: Some("128Mi".to_string()), // Default request
                ephemeral_storage: None,
            },
        }
    }

    fn count_running_pods(&self) -> usize {
        let pod_metrics = self.pod_metrics.lock().unwrap();
        pod_metrics.values()
            .filter(|pod| !pod.containers.is_empty())
            .count()
    }

    fn parse_cpu_request(&self, cpu_str: &Option<String>) -> f64 {
        match cpu_str {
            Some(cpu) => {
                if cpu.ends_with('m') {
                    cpu.trim_end_matches('m').parse::<f64>().unwrap_or(0.0)
                } else {
                    cpu.parse::<f64>().unwrap_or(0.0) * 1000.0 // Convert cores to millicores
                }
            }
            None => 100.0, // Default 100m
        }
    }

    fn parse_memory_request(&self, memory_str: &Option<String>) -> f64 {
        match memory_str {
            Some(memory) => {
                let memory = memory.trim();
                if memory.ends_with("Mi") {
                    memory.trim_end_matches("Mi").parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0
                } else if memory.ends_with("Gi") {
                    memory.trim_end_matches("Gi").parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0 * 1024.0
                } else {
                    memory.parse::<f64>().unwrap_or(0.0)
                }
            }
            None => 128.0 * 1024.0 * 1024.0, // Default 128Mi
        }
    }

    fn parse_cpu_capacity(&self, cpu_str: &str) -> f64 {
        cpu_str.parse::<f64>().unwrap_or(0.0)
    }

    fn parse_memory_capacity(&self, memory_str: &str) -> u64 {
        if memory_str.ends_with("Gi") {
            let value = memory_str.trim_end_matches("Gi").parse::<f64>().unwrap_or(0.0);
            (value * 1024.0 * 1024.0 * 1024.0) as u64
        } else {
            memory_str.parse::<u64>().unwrap_or(0)
        }
    }
}

impl Default for ClusterMetrics {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            ready_nodes: 0,
            total_pods: 0,
            running_pods: 0,
            pending_pods: 0,
            failed_pods: 0,
            cluster_resource_usage: ClusterResourceUsage {
                cpu_usage_cores: 0.0,
                memory_usage_bytes: 0,
                storage_usage_bytes: 0,
                network_usage_bytes_per_sec: 0,
            },
            cluster_resource_capacity: ClusterResourceCapacity {
                cpu_capacity_cores: 0.0,
                memory_capacity_bytes: 0,
                storage_capacity_bytes: 0,
                pod_capacity: 0,
            },
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Metrics initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Metrics collection failed: {0}")]
    CollectionFailed(String),
    #[error("Metrics parsing failed: {0}")]
    ParsingFailed(String),
    #[error("Metrics aggregation failed: {0}")]
    AggregationFailed(String),
}