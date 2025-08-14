use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration, Instant};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::scale::scale::{Pod, Service, Deployment, Node};

/// Comprehensive autoscaling system for pods, deployments, and cluster nodes
#[derive(Clone)]
pub struct AutoscalerManager {
    pub hpa_controller: Arc<HorizontalPodAutoscaler>,
    pub vpa_controller: Arc<VerticalPodAutoscaler>,
    pub cluster_autoscaler: Arc<ClusterAutoscaler>,
    pub custom_metrics: Arc<Mutex<HashMap<String, CustomMetric>>>,
    pub config: AutoscalerConfig,
}

#[derive(Debug, Clone)]
pub struct AutoscalerConfig {
    pub hpa_sync_interval: Duration,
    pub vpa_sync_interval: Duration,
    pub cluster_sync_interval: Duration,
    pub metrics_retention_period: Duration,
    pub scale_down_delay: Duration,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub max_scale_up_rate: f64,
    pub max_scale_down_rate: f64,
}

impl Default for AutoscalerConfig {
    fn default() -> Self {
        Self {
            hpa_sync_interval: Duration::from_secs(15),
            vpa_sync_interval: Duration::from_secs(30),
            cluster_sync_interval: Duration::from_secs(60),
            metrics_retention_period: Duration::from_secs(3600), // 1 hour
            scale_down_delay: Duration::from_secs(300), // 5 minutes
            scale_up_threshold: 0.7,   // 70%
            scale_down_threshold: 0.3, // 30%
            max_scale_up_rate: 2.0,    // 2x per scaling event
            max_scale_down_rate: 0.5,  // 0.5x per scaling event
        }
    }
}

/// Horizontal Pod Autoscaler - scales pod replicas based on metrics
#[derive(Clone)]
pub struct HorizontalPodAutoscaler {
    pub hpa_specs: Arc<Mutex<HashMap<String, HpaSpec>>>,
    pub scaling_history: Arc<Mutex<Vec<ScalingEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpaSpec {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub target_ref: ScaleTargetRef,
    pub min_replicas: i32,
    pub max_replicas: i32,
    pub metrics: Vec<MetricSpec>,
    pub behavior: Option<HpaScalingBehavior>,
    pub created_at: DateTime<Utc>,
    pub status: HpaStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleTargetRef {
    pub api_version: String,
    pub kind: String, // "Deployment", "ReplicaSet", etc.
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSpec {
    pub metric_type: MetricType,
    pub resource: Option<ResourceMetricSpec>,
    pub pods: Option<PodsMetricSpec>,
    pub object: Option<ObjectMetricSpec>,
    pub external: Option<ExternalMetricSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Resource,
    Pods,
    Object,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetricSpec {
    pub name: String, // "cpu", "memory"
    pub target: MetricTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodsMetricSpec {
    pub metric: MetricIdentifier,
    pub target: MetricTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetricSpec {
    pub metric: MetricIdentifier,
    pub target: MetricTarget,
    pub described_object: CrossVersionObjectReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMetricSpec {
    pub metric: MetricIdentifier,
    pub target: MetricTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricIdentifier {
    pub name: String,
    pub selector: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTarget {
    pub target_type: MetricTargetType,
    pub value: Option<String>,
    pub average_value: Option<String>,
    pub average_utilization: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricTargetType {
    Utilization,
    Value,
    AverageValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossVersionObjectReference {
    pub api_version: String,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpaScalingBehavior {
    pub scale_up: Option<HPAScalingRules>,
    pub scale_down: Option<HPAScalingRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPAScalingRules {
    pub stabilization_window_seconds: Option<i32>,
    pub select_policy: Option<ScalingPolicySelect>,
    pub policies: Vec<HPAScalingPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingPolicySelect {
    Max,
    Min,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPAScalingPolicy {
    pub policy_type: HPAScalingPolicyType,
    pub value: i32,
    pub period_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HPAScalingPolicyType {
    Pods,
    Percent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpaStatus {
    pub current_replicas: i32,
    pub desired_replicas: i32,
    pub current_metrics: Vec<MetricStatus>,
    pub conditions: Vec<HpaCondition>,
    pub last_scale_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStatus {
    pub metric_type: MetricType,
    pub resource: Option<ResourceMetricStatus>,
    pub pods: Option<PodsMetricStatus>,
    pub object: Option<ObjectMetricStatus>,
    pub external: Option<ExternalMetricStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetricStatus {
    pub name: String,
    pub current: MetricValueStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodsMetricStatus {
    pub metric: MetricIdentifier,
    pub current: MetricValueStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetricStatus {
    pub metric: MetricIdentifier,
    pub current: MetricValueStatus,
    pub described_object: CrossVersionObjectReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMetricStatus {
    pub metric: MetricIdentifier,
    pub current: MetricValueStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValueStatus {
    pub value: Option<String>,
    pub average_value: Option<String>,
    pub average_utilization: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpaCondition {
    pub condition_type: String,
    pub status: String,
    pub last_transition_time: DateTime<Utc>,
    pub reason: String,
    pub message: String,
}

/// Vertical Pod Autoscaler - adjusts resource requests and limits
#[derive(Clone)]
pub struct VerticalPodAutoscaler {
    pub vpa_specs: Arc<Mutex<HashMap<String, VpaSpec>>>,
    pub recommendations: Arc<Mutex<HashMap<String, VpaRecommendation>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaSpec {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub target_ref: ScaleTargetRef,
    pub update_policy: VpaUpdatePolicy,
    pub resource_policy: Option<VpaResourcePolicy>,
    pub created_at: DateTime<Utc>,
    pub status: VpaStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaUpdatePolicy {
    pub update_mode: VpaUpdateMode,
    pub min_replicas: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpaUpdateMode {
    Off,
    Initial,
    Recreation,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaResourcePolicy {
    pub container_policies: Vec<VpaContainerResourcePolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaContainerResourcePolicy {
    pub container_name: String,
    pub mode: Option<VpaContainerScalingMode>,
    pub min_allowed: Option<HashMap<String, String>>,
    pub max_allowed: Option<HashMap<String, String>>,
    pub controlled_resources: Option<Vec<String>>,
    pub controlled_values: Option<VpaControlledValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpaContainerScalingMode {
    Auto,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpaControlledValues {
    RequestsAndLimits,
    RequestsOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaStatus {
    pub conditions: Vec<VpaCondition>,
    pub recommendation: Option<VpaRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaCondition {
    pub condition_type: String,
    pub status: String,
    pub last_transition_time: DateTime<Utc>,
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaRecommendation {
    pub container_recommendations: Vec<VpaContainerRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpaContainerRecommendation {
    pub container_name: String,
    pub target: HashMap<String, String>,
    pub lower_bound: HashMap<String, String>,
    pub upper_bound: HashMap<String, String>,
    pub uncapped_target: HashMap<String, String>,
}

/// Cluster Autoscaler - scales worker nodes based on resource demand
#[derive(Clone)]
pub struct ClusterAutoscaler {
    pub node_groups: Arc<Mutex<HashMap<String, NodeGroup>>>,
    pub scaling_activities: Arc<Mutex<Vec<ClusterScalingActivity>>>,
    pub unschedulable_pods: Arc<Mutex<Vec<String>>>, // Pod IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    pub id: String,
    pub name: String,
    pub min_size: i32,
    pub max_size: i32,
    pub desired_capacity: i32,
    pub instance_type: String,
    pub availability_zones: Vec<String>,
    pub labels: HashMap<String, String>,
    pub taints: Vec<NodeTaint>,
    pub auto_scaling_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub status: NodeGroupStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTaint {
    pub key: String,
    pub value: Option<String>,
    pub effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintEffect {
    NoSchedule,
    PreferNoSchedule,
    NoExecute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroupStatus {
    pub ready_nodes: i32,
    pub total_nodes: i32,
    pub conditions: Vec<NodeGroupCondition>,
    pub last_scale_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroupCondition {
    pub condition_type: String,
    pub status: String,
    pub last_transition_time: DateTime<Utc>,
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterScalingActivity {
    pub id: String,
    pub activity_type: ClusterScalingActivityType,
    pub node_group_name: String,
    pub description: String,
    pub cause: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status_code: ClusterScalingActivityStatus,
    pub status_message: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterScalingActivityType {
    ScaleUp,
    ScaleDown,
    InstanceTermination,
    InstanceLaunch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterScalingActivityStatus {
    InProgress,
    Successful,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingEvent {
    pub id: String,
    pub event_type: ScalingEventType,
    pub target_ref: ScaleTargetRef,
    pub old_replicas: i32,
    pub new_replicas: i32,
    pub reason: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metrics_snapshot: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingEventType {
    HorizontalScaleUp,
    HorizontalScaleDown,
    VerticalScaleUp,
    VerticalScaleDown,
    ClusterScaleUp,
    ClusterScaleDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub name: String,
    pub namespace: String,
    pub value: f64,
    pub unit: String,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub ttl: Duration,
}

impl AutoscalerManager {
    pub fn new(config: AutoscalerConfig) -> Self {
        Self {
            hpa_controller: Arc::new(HorizontalPodAutoscaler::new()),
            vpa_controller: Arc::new(VerticalPodAutoscaler::new()),
            cluster_autoscaler: Arc::new(ClusterAutoscaler::new()),
            custom_metrics: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Start all autoscaling controllers
    pub async fn start(&self) {
        println!("Starting autoscaler manager with config: {:?}", self.config);

        // Start HPA controller
        let hpa_controller = self.hpa_controller.clone();
        let hpa_interval = self.config.hpa_sync_interval;
        tokio::spawn(async move {
            hpa_controller.start_controller(hpa_interval).await;
        });

        // Start VPA controller
        let vpa_controller = self.vpa_controller.clone();
        let vpa_interval = self.config.vpa_sync_interval;
        tokio::spawn(async move {
            vpa_controller.start_controller(vpa_interval).await;
        });

        // Start Cluster Autoscaler
        let cluster_autoscaler = self.cluster_autoscaler.clone();
        let cluster_interval = self.config.cluster_sync_interval;
        tokio::spawn(async move {
            cluster_autoscaler.start_controller(cluster_interval).await;
        });

        // Start metrics cleanup
        let custom_metrics = self.custom_metrics.clone();
        let retention_period = self.config.metrics_retention_period;
        tokio::spawn(async move {
            Self::cleanup_expired_metrics(custom_metrics, retention_period).await;
        });

        println!("All autoscaling controllers started");
    }

    /// Register a custom metric for scaling decisions
    pub fn register_custom_metric(&self, metric: CustomMetric) {
        let mut metrics = self.custom_metrics.lock().unwrap();
        metrics.insert(metric.name.clone(), metric);
    }

    /// Get autoscaling statistics
    pub fn get_autoscaling_stats(&self) -> AutoscalingStats {
        let hpa_count = self.hpa_controller.hpa_specs.lock().unwrap().len();
        let vpa_count = self.vpa_controller.vpa_specs.lock().unwrap().len();
        let node_groups_count = self.cluster_autoscaler.node_groups.lock().unwrap().len();
        let scaling_events_count = self.hpa_controller.scaling_history.lock().unwrap().len();
        let custom_metrics_count = self.custom_metrics.lock().unwrap().len();

        let total_nodes: i32 = self.cluster_autoscaler.node_groups.lock().unwrap()
            .values()
            .map(|ng| ng.status.total_nodes)
            .sum();

        let ready_nodes: i32 = self.cluster_autoscaler.node_groups.lock().unwrap()
            .values()
            .map(|ng| ng.status.ready_nodes)
            .sum();

        AutoscalingStats {
            hpa_count,
            vpa_count,
            node_groups_count,
            total_nodes: total_nodes as usize,
            ready_nodes: ready_nodes as usize,
            scaling_events_count,
            custom_metrics_count,
        }
    }

    /// Cleanup expired custom metrics
    async fn cleanup_expired_metrics(
        custom_metrics: Arc<Mutex<HashMap<String, CustomMetric>>>,
        retention_period: Duration,
    ) {
        let mut interval = interval(Duration::from_secs(300)); // Check every 5 minutes

        loop {
            interval.tick().await;

            let now = Utc::now();
            let mut metrics = custom_metrics.lock().unwrap();
            
            metrics.retain(|_, metric| {
                let age = now.signed_duration_since(metric.timestamp);
                age.to_std().unwrap_or(Duration::ZERO) < retention_period
            });
        }
    }
}

impl HorizontalPodAutoscaler {
    pub fn new() -> Self {
        Self {
            hpa_specs: Arc::new(Mutex::new(HashMap::new())),
            scaling_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start the HPA controller loop
    pub async fn start_controller(&self, sync_interval: Duration) {
        let mut interval = interval(sync_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.reconcile_hpas().await {
                eprintln!("HPA reconciliation error: {}", e);
            }
        }
    }

    /// Create a new HPA spec
    pub fn create_hpa(&self, mut spec: HpaSpec) -> Result<(), AutoscalerError> {
        spec.id = Uuid::new_v4().to_string();
        spec.created_at = Utc::now();
        spec.status = HpaStatus {
            current_replicas: 0,
            desired_replicas: spec.min_replicas,
            current_metrics: vec![],
            conditions: vec![],
            last_scale_time: None,
        };

        let mut hpas = self.hpa_specs.lock().unwrap();
        hpas.insert(spec.id.clone(), spec.clone());

        println!("Created HPA: {} for target: {}", spec.name, spec.target_ref.name);
        Ok(())
    }

    /// Reconcile all HPA specs
    async fn reconcile_hpas(&self) -> Result<(), AutoscalerError> {
        let hpa_specs: Vec<HpaSpec> = {
            let hpas = self.hpa_specs.lock().unwrap();
            hpas.values().cloned().collect()
        };

        for mut hpa_spec in hpa_specs {
            if let Err(e) = self.reconcile_single_hpa(&mut hpa_spec).await {
                eprintln!("Failed to reconcile HPA {}: {}", hpa_spec.name, e);
                continue;
            }

            // Update the spec in storage
            let mut hpas = self.hpa_specs.lock().unwrap();
            hpas.insert(hpa_spec.id.clone(), hpa_spec);
        }

        Ok(())
    }

    /// Reconcile a single HPA spec
    async fn reconcile_single_hpa(&self, hpa_spec: &mut HpaSpec) -> Result<(), AutoscalerError> {
        // Get current metrics
        let current_metrics = self.collect_metrics_for_hpa(hpa_spec).await?;
        
        // Calculate desired replicas based on metrics
        let desired_replicas = self.calculate_desired_replicas(hpa_spec, &current_metrics)?;
        
        // Apply scaling behavior constraints
        let constrained_replicas = self.apply_scaling_behavior(hpa_spec, desired_replicas);
        
        // Check if scaling is needed
        if constrained_replicas != hpa_spec.status.current_replicas {
            self.scale_target(hpa_spec, constrained_replicas).await?;
            
            // Record scaling event
            let scaling_event = ScalingEvent {
                id: Uuid::new_v4().to_string(),
                event_type: if constrained_replicas > hpa_spec.status.current_replicas {
                    ScalingEventType::HorizontalScaleUp
                } else {
                    ScalingEventType::HorizontalScaleDown
                },
                target_ref: hpa_spec.target_ref.clone(),
                old_replicas: hpa_spec.status.current_replicas,
                new_replicas: constrained_replicas,
                reason: "MetricThresholdReached".to_string(),
                message: format!("Scaled from {} to {} replicas", 
                                hpa_spec.status.current_replicas, constrained_replicas),
                timestamp: Utc::now(),
                metrics_snapshot: current_metrics.iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect(),
            };
            
            let mut history = self.scaling_history.lock().unwrap();
            history.push(scaling_event);
            
            // Update HPA status
            hpa_spec.status.current_replicas = constrained_replicas;
            hpa_spec.status.desired_replicas = constrained_replicas;
            hpa_spec.status.last_scale_time = Some(Utc::now());
        }
        
        // Update current metrics in status
        hpa_spec.status.current_metrics = self.convert_to_metric_status(&current_metrics);

        Ok(())
    }

    /// Collect metrics for an HPA spec
    async fn collect_metrics_for_hpa(&self, hpa_spec: &HpaSpec) -> Result<HashMap<String, f64>, AutoscalerError> {
        let mut metrics = HashMap::new();

        for metric_spec in &hpa_spec.metrics {
            match &metric_spec.metric_type {
                MetricType::Resource => {
                    if let Some(resource_spec) = &metric_spec.resource {
                        let value = self.collect_resource_metric(&resource_spec.name, hpa_spec).await?;
                        metrics.insert(resource_spec.name.clone(), value);
                    }
                }
                MetricType::Pods => {
                    // Mock pods metric collection
                    metrics.insert("pods_metric".to_string(), 50.0);
                }
                MetricType::Object => {
                    // Mock object metric collection
                    metrics.insert("object_metric".to_string(), 75.0);
                }
                MetricType::External => {
                    // Mock external metric collection
                    metrics.insert("external_metric".to_string(), 60.0);
                }
            }
        }

        Ok(metrics)
    }

    /// Collect resource metrics (CPU, memory) for a target
    async fn collect_resource_metric(&self, resource_name: &str, hpa_spec: &HpaSpec) -> Result<f64, AutoscalerError> {
        // Mock metric collection - in reality this would query metrics from pods
        match resource_name {
            "cpu" => {
                // Simulate CPU utilization between 20-90%
                let utilization = 30.0 + (rand::random::<f64>() * 60.0);
                Ok(utilization)
            }
            "memory" => {
                // Simulate memory utilization between 25-85%
                let utilization = 25.0 + (rand::random::<f64>() * 60.0);
                Ok(utilization)
            }
            _ => Ok(50.0), // Default utilization
        }
    }

    /// Calculate desired replicas based on metrics
    fn calculate_desired_replicas(&self, hpa_spec: &HpaSpec, metrics: &HashMap<String, f64>) -> Result<i32, AutoscalerError> {
        let current_replicas = hpa_spec.status.current_replicas.max(1) as f64;
        let mut scale_ratios = Vec::new();

        for metric_spec in &hpa_spec.metrics {
            if let MetricType::Resource = metric_spec.metric_type {
                if let Some(resource_spec) = &metric_spec.resource {
                    if let Some(current_value) = metrics.get(&resource_spec.name) {
                        if let Some(target_utilization) = resource_spec.target.average_utilization {
                            let ratio = current_value / target_utilization as f64;
                            scale_ratios.push(ratio);
                        }
                    }
                }
            }
        }

        if scale_ratios.is_empty() {
            return Ok(hpa_spec.status.current_replicas);
        }

        // Use the maximum ratio to ensure all metrics are within bounds
        let max_ratio = scale_ratios.iter().fold(0.0_f64, |a, &b| a.max(b));
        let desired_replicas = (current_replicas * max_ratio).ceil() as i32;

        // Apply min/max constraints
        let constrained_replicas = desired_replicas
            .max(hpa_spec.min_replicas)
            .min(hpa_spec.max_replicas);

        Ok(constrained_replicas)
    }

    /// Apply scaling behavior constraints
    fn apply_scaling_behavior(&self, hpa_spec: &HpaSpec, desired_replicas: i32) -> i32 {
        let current_replicas = hpa_spec.status.current_replicas;
        
        // Apply rate limiting based on scaling behavior
        if let Some(behavior) = &hpa_spec.behavior {
            // Check scale up policies
            if desired_replicas > current_replicas {
                if let Some(scale_up) = &behavior.scale_up {
                    return self.apply_scaling_policies(&scale_up.policies, current_replicas, desired_replicas, true);
                }
            }
            // Check scale down policies
            else if desired_replicas < current_replicas {
                if let Some(scale_down) = &behavior.scale_down {
                    return self.apply_scaling_policies(&scale_down.policies, current_replicas, desired_replicas, false);
                }
            }
        }

        desired_replicas
    }

    /// Apply scaling policies to limit rate of change
    fn apply_scaling_policies(&self, policies: &[HPAScalingPolicy], current: i32, desired: i32, scale_up: bool) -> i32 {
        let mut max_change = if scale_up { desired - current } else { current - desired };

        for policy in policies {
            let change_limit = match policy.policy_type {
                HPAScalingPolicyType::Pods => policy.value,
                HPAScalingPolicyType::Percent => {
                    ((current as f64 * policy.value as f64) / 100.0) as i32
                }
            };

            max_change = max_change.min(change_limit);
        }

        if scale_up {
            current + max_change
        } else {
            current - max_change
        }
    }

    /// Scale the target deployment/replicaset
    async fn scale_target(&self, hpa_spec: &HpaSpec, replicas: i32) -> Result<(), AutoscalerError> {
        println!("Scaling {} {} to {} replicas", 
                 hpa_spec.target_ref.kind, 
                 hpa_spec.target_ref.name, 
                 replicas);

        // Mock scaling operation - in reality this would update the deployment spec
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Convert metrics to status format
    fn convert_to_metric_status(&self, metrics: &HashMap<String, f64>) -> Vec<MetricStatus> {
        metrics.iter().map(|(name, value)| {
            MetricStatus {
                metric_type: MetricType::Resource,
                resource: Some(ResourceMetricStatus {
                    name: name.clone(),
                    current: MetricValueStatus {
                        value: None,
                        average_value: None,
                        average_utilization: Some(*value as i32),
                    },
                }),
                pods: None,
                object: None,
                external: None,
            }
        }).collect()
    }
}

impl VerticalPodAutoscaler {
    pub fn new() -> Self {
        Self {
            vpa_specs: Arc::new(Mutex::new(HashMap::new())),
            recommendations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the VPA controller loop
    pub async fn start_controller(&self, sync_interval: Duration) {
        let mut interval = interval(sync_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.reconcile_vpas().await {
                eprintln!("VPA reconciliation error: {}", e);
            }
        }
    }

    /// Create a new VPA spec
    pub fn create_vpa(&self, mut spec: VpaSpec) -> Result<(), AutoscalerError> {
        spec.id = Uuid::new_v4().to_string();
        spec.created_at = Utc::now();
        spec.status = VpaStatus {
            conditions: vec![],
            recommendation: None,
        };

        let mut vpas = self.vpa_specs.lock().unwrap();
        vpas.insert(spec.id.clone(), spec.clone());

        println!("Created VPA: {} for target: {}", spec.name, spec.target_ref.name);
        Ok(())
    }

    /// Reconcile all VPA specs
    async fn reconcile_vpas(&self) -> Result<(), AutoscalerError> {
        let vpa_specs: Vec<VpaSpec> = {
            let vpas = self.vpa_specs.lock().unwrap();
            vpas.values().cloned().collect()
        };

        for mut vpa_spec in vpa_specs {
            if let Err(e) = self.reconcile_single_vpa(&mut vpa_spec).await {
                eprintln!("Failed to reconcile VPA {}: {}", vpa_spec.name, e);
                continue;
            }

            let mut vpas = self.vpa_specs.lock().unwrap();
            vpas.insert(vpa_spec.id.clone(), vpa_spec);
        }

        Ok(())
    }

    /// Reconcile a single VPA spec
    async fn reconcile_single_vpa(&self, vpa_spec: &mut VpaSpec) -> Result<(), AutoscalerError> {
        // Generate resource recommendations based on historical usage
        let recommendation = self.generate_recommendations(vpa_spec).await?;
        
        // Update VPA status
        vpa_spec.status.recommendation = Some(recommendation.clone());
        
        // Apply recommendations based on update mode
        match vpa_spec.update_policy.update_mode {
            VpaUpdateMode::Auto => {
                self.apply_recommendations(vpa_spec, &recommendation).await?;
            }
            VpaUpdateMode::Recreation => {
                // Would recreate pods with new resource specs
                println!("VPA {} would recreate pods with new resource specs", vpa_spec.name);
            }
            VpaUpdateMode::Initial => {
                // Only applies to new pods
                println!("VPA {} recommendations apply to new pods only", vpa_spec.name);
            }
            VpaUpdateMode::Off => {
                // Only generates recommendations, no action
            }
        }

        Ok(())
    }

    /// Generate resource recommendations for a VPA target
    async fn generate_recommendations(&self, vpa_spec: &VpaSpec) -> Result<VpaRecommendation, AutoscalerError> {
        // Mock recommendation generation - in reality this would analyze historical metrics
        let container_recommendations = vec![
            VpaContainerRecommendation {
                container_name: "main".to_string(),
                target: {
                    let mut target = HashMap::new();
                    target.insert("cpu".to_string(), "200m".to_string());
                    target.insert("memory".to_string(), "256Mi".to_string());
                    target
                },
                lower_bound: {
                    let mut lower = HashMap::new();
                    lower.insert("cpu".to_string(), "100m".to_string());
                    lower.insert("memory".to_string(), "128Mi".to_string());
                    lower
                },
                upper_bound: {
                    let mut upper = HashMap::new();
                    upper.insert("cpu".to_string(), "500m".to_string());
                    upper.insert("memory".to_string(), "512Mi".to_string());
                    upper
                },
                uncapped_target: {
                    let mut uncapped = HashMap::new();
                    uncapped.insert("cpu".to_string(), "250m".to_string());
                    uncapped.insert("memory".to_string(), "300Mi".to_string());
                    uncapped
                },
            }
        ];

        Ok(VpaRecommendation {
            container_recommendations,
        })
    }

    /// Apply VPA recommendations to the target
    async fn apply_recommendations(&self, vpa_spec: &VpaSpec, recommendation: &VpaRecommendation) -> Result<(), AutoscalerError> {
        for container_rec in &recommendation.container_recommendations {
            println!("Applying VPA recommendations for container {}: CPU={}, Memory={}", 
                     container_rec.container_name,
                     container_rec.target.get("cpu").unwrap_or(&"unknown".to_string()),
                     container_rec.target.get("memory").unwrap_or(&"unknown".to_string()));
        }

        // Mock application - in reality this would update pod specs
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(())
    }
}

impl ClusterAutoscaler {
    pub fn new() -> Self {
        Self {
            node_groups: Arc::new(Mutex::new(HashMap::new())),
            scaling_activities: Arc::new(Mutex::new(Vec::new())),
            unschedulable_pods: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start the cluster autoscaler controller loop
    pub async fn start_controller(&self, sync_interval: Duration) {
        let mut interval = interval(sync_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.reconcile_cluster_scaling().await {
                eprintln!("Cluster autoscaler reconciliation error: {}", e);
            }
        }
    }

    /// Create a new node group
    pub fn create_node_group(&self, mut node_group: NodeGroup) -> Result<(), AutoscalerError> {
        node_group.id = Uuid::new_v4().to_string();
        node_group.created_at = Utc::now();
        node_group.status = NodeGroupStatus {
            ready_nodes: 0,
            total_nodes: 0,
            conditions: vec![],
            last_scale_time: None,
        };

        let mut node_groups = self.node_groups.lock().unwrap();
        node_groups.insert(node_group.id.clone(), node_group.clone());

        println!("Created node group: {} (min: {}, max: {})", 
                 node_group.name, node_group.min_size, node_group.max_size);
        Ok(())
    }

    /// Reconcile cluster scaling decisions
    async fn reconcile_cluster_scaling(&self) -> Result<(), AutoscalerError> {
        // Check for unschedulable pods that need more nodes
        self.check_scale_up_needed().await?;
        
        // Check for underutilized nodes that can be removed
        self.check_scale_down_possible().await?;

        Ok(())
    }

    /// Check if scale up is needed due to unschedulable pods
    async fn check_scale_up_needed(&self) -> Result<(), AutoscalerError> {
        let unschedulable_pods = self.unschedulable_pods.lock().unwrap().len();
        
        if unschedulable_pods > 0 {
            println!("Found {} unschedulable pods, checking for scale up", unschedulable_pods);
            
            let node_groups: Vec<NodeGroup> = {
                let groups = self.node_groups.lock().unwrap();
                groups.values().cloned().collect()
            };

            for node_group in node_groups {
                if node_group.auto_scaling_enabled && node_group.status.total_nodes < node_group.max_size {
                    let desired_nodes = (node_group.status.total_nodes + 1).min(node_group.max_size);
                    self.scale_node_group(&node_group.id, desired_nodes, ClusterScalingActivityType::ScaleUp).await?;
                    break; // Scale one node group at a time
                }
            }
        }

        Ok(())
    }

    /// Check if scale down is possible due to low utilization
    async fn check_scale_down_possible(&self) -> Result<(), AutoscalerError> {
        let node_groups: Vec<NodeGroup> = {
            let groups = self.node_groups.lock().unwrap();
            groups.values().cloned().collect()
        };

        for node_group in node_groups {
            if node_group.auto_scaling_enabled && node_group.status.total_nodes > node_group.min_size {
                // Mock utilization check - in reality this would check actual node metrics
                let utilization = rand::random::<f64>();
                
                if utilization < 0.3 { // Less than 30% utilization
                    let desired_nodes = (node_group.status.total_nodes - 1).max(node_group.min_size);
                    self.scale_node_group(&node_group.id, desired_nodes, ClusterScalingActivityType::ScaleDown).await?;
                    break; // Scale one node group at a time
                }
            }
        }

        Ok(())
    }

    /// Scale a node group to the desired size
    async fn scale_node_group(&self, node_group_id: &str, desired_nodes: i32, activity_type: ClusterScalingActivityType) -> Result<(), AutoscalerError> {
        let mut node_groups = self.node_groups.lock().unwrap();
        
        if let Some(node_group) = node_groups.get_mut(node_group_id) {
            let old_size = node_group.status.total_nodes;
            node_group.status.total_nodes = desired_nodes;
            node_group.status.last_scale_time = Some(Utc::now());

            // Record scaling activity
            let activity = ClusterScalingActivity {
                id: Uuid::new_v4().to_string(),
                activity_type,
                node_group_name: node_group.name.clone(),
                description: format!("Scaling from {} to {} nodes", old_size, desired_nodes),
                cause: "ResourceDemand".to_string(),
                start_time: Utc::now(),
                end_time: None,
                status_code: ClusterScalingActivityStatus::InProgress,
                status_message: "Scaling in progress".to_string(),
                details: HashMap::new(),
            };

            let mut activities = self.scaling_activities.lock().unwrap();
            activities.push(activity);

            println!("Scaling node group {} from {} to {} nodes", 
                     node_group.name, old_size, desired_nodes);

            // Mock scaling delay
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Update to successful
            if let Some(last_activity) = activities.last_mut() {
                last_activity.end_time = Some(Utc::now());
                last_activity.status_code = ClusterScalingActivityStatus::Successful;
                last_activity.status_message = "Scaling completed successfully".to_string();
            }

            // Update ready nodes (mock - assume all nodes become ready)
            node_group.status.ready_nodes = desired_nodes;
        }

        Ok(())
    }

    /// Report an unschedulable pod
    pub fn report_unschedulable_pod(&self, pod_id: String) {
        let mut unschedulable_pods = self.unschedulable_pods.lock().unwrap();
        if !unschedulable_pods.contains(&pod_id) {
            unschedulable_pods.push(pod_id);
        }
    }

    /// Remove a pod from unschedulable list (when it gets scheduled)
    pub fn remove_unschedulable_pod(&self, pod_id: &str) {
        let mut unschedulable_pods = self.unschedulable_pods.lock().unwrap();
        unschedulable_pods.retain(|id| id != pod_id);
    }
}

#[derive(Debug, Clone)]
pub struct AutoscalingStats {
    pub hpa_count: usize,
    pub vpa_count: usize,
    pub node_groups_count: usize,
    pub total_nodes: usize,
    pub ready_nodes: usize,
    pub scaling_events_count: usize,
    pub custom_metrics_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum AutoscalerError {
    #[error("Metric collection failed: {0}")]
    MetricCollectionFailed(String),
    #[error("Scaling operation failed: {0}")]
    ScalingFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Target not found: {0}")]
    TargetNotFound(String),
    #[error("Resource constraint violation: {0}")]
    ResourceConstraintViolation(String),
}