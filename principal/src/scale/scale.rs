use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::providers::local::models::request::container::{LocalContainer, CreateContainerRequest};
use crate::providers::fly::models::request::machine::{FlyMachine, CreateMachineRequest, MachineConfig, GuestConfig};

/// Core scheduling and scaling logic for Infralink
#[derive(Debug, Clone)]
pub struct InfralinkScheduler {
    pub nodes: Vec<Node>,
    pub pods: HashMap<String, Pod>,
    pub deployments: HashMap<String, Deployment>,
    pub services: HashMap<String, Service>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub provider: CloudProvider,
    pub status: NodeStatus,
    pub capacity: ResourceSpec,
    pub allocatable: ResourceSpec,
    pub allocated: ResourceSpec,
    pub labels: HashMap<String, String>,
    pub taints: Vec<Taint>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    Local,
    Fly { app_name: String },
    Vultr { region: String },
    Hetzner { datacenter: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Ready,
    NotReady,
    Unknown,
    Cordoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_cores: f64,
    pub memory_mb: u64,
    pub storage_gb: u64,
    pub network_mbps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taint {
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
pub struct Pod {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub spec: PodSpec,
    pub status: PodStatus,
    pub node_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSpec {
    pub containers: Vec<ContainerSpec>,
    pub restart_policy: RestartPolicy,
    pub resources: ResourceRequirements,
    pub node_selector: HashMap<String, String>,
    pub tolerations: Vec<Toleration>,
    pub affinity: Option<Affinity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub command: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub env: Vec<EnvVar>,
    pub ports: Vec<ContainerPort>,
    pub resources: ResourceRequirements,
    pub volume_mounts: Vec<VolumeMount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: Option<String>,
    pub value_from: Option<EnvVarSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvVarSource {
    ConfigMapKeyRef { name: String, key: String },
    SecretKeyRef { name: String, key: String },
    FieldRef { field_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPort {
    pub name: Option<String>,
    pub container_port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub requests: ResourceSpec,
    pub limits: ResourceSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    pub key: String,
    pub operator: TolerationOperator,
    pub value: Option<String>,
    pub effect: Option<TaintEffect>,
    pub toleration_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TolerationOperator {
    Exists,
    Equal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affinity {
    pub node_affinity: Option<NodeAffinity>,
    pub pod_affinity: Option<PodAffinity>,
    pub pod_anti_affinity: Option<PodAffinity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAffinity {
    pub required: Option<NodeSelector>,
    pub preferred: Vec<PreferredSchedulingTerm>,
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
    pub operator: NodeSelectorOperator,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeSelectorOperator {
    In,
    NotIn,
    Exists,
    DoesNotExist,
    Gt,
    Lt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferredSchedulingTerm {
    pub weight: i32,
    pub preference: NodeSelectorTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinity {
    pub required: Vec<PodAffinityTerm>,
    pub preferred: Vec<WeightedPodAffinityTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinityTerm {
    pub label_selector: Option<LabelSelector>,
    pub topology_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedPodAffinityTerm {
    pub weight: i32,
    pub pod_affinity_term: PodAffinityTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    pub match_labels: HashMap<String, String>,
    pub match_expressions: Vec<LabelSelectorRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    pub key: String,
    pub operator: LabelSelectorOperator,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelSelectorOperator {
    In,
    NotIn,
    Exists,
    DoesNotExist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
    pub phase: PodPhase,
    pub conditions: Vec<PodCondition>,
    pub container_statuses: Vec<ContainerStatus>,
    pub pod_ip: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PodPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodCondition {
    pub condition_type: PodConditionType,
    pub status: ConditionStatus,
    pub last_probe_time: Option<DateTime<Utc>>,
    pub last_transition_time: DateTime<Utc>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PodConditionType {
    PodScheduled,
    PodReady,
    Initialized,
    ContainersReady,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub name: String,
    pub state: ContainerState,
    pub ready: bool,
    pub restart_count: u32,
    pub image: String,
    pub image_id: String,
    pub container_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerState {
    Waiting { reason: Option<String>, message: Option<String> },
    Running { started_at: DateTime<Utc> },
    Terminated { exit_code: i32, reason: Option<String>, started_at: Option<DateTime<Utc>>, finished_at: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub spec: DeploymentSpec,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSpec {
    pub replicas: u32,
    pub selector: LabelSelector,
    pub template: PodTemplate,
    pub strategy: DeploymentStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodTemplate {
    pub metadata: PodTemplateMetadata,
    pub spec: PodSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodTemplateMetadata {
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStrategy {
    pub strategy_type: DeploymentStrategyType,
    pub rolling_update: Option<RollingUpdateDeployment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategyType {
    RollingUpdate,
    Recreate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingUpdateDeployment {
    pub max_unavailable: Option<IntOrString>,
    pub max_surge: Option<IntOrString>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntOrString {
    Int(u32),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub replicas: u32,
    pub updated_replicas: u32,
    pub ready_replicas: u32,
    pub available_replicas: u32,
    pub conditions: Vec<DeploymentCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCondition {
    pub condition_type: DeploymentConditionType,
    pub status: ConditionStatus,
    pub last_update_time: DateTime<Utc>,
    pub last_transition_time: DateTime<Utc>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentConditionType {
    Available,
    Progressing,
    ReplicaFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub spec: ServiceSpec,
    pub status: ServiceStatus,
    pub created_at: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub selector: HashMap<String, String>,
    pub ports: Vec<ServicePort>,
    pub service_type: ServiceType,
    pub cluster_ip: Option<String>,
    pub external_ips: Vec<String>,
    pub load_balancer_ip: Option<String>,
    pub session_affinity: SessionAffinity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub protocol: String,
    pub port: u16,
    pub target_port: IntOrString,
    pub node_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionAffinity {
    None,
    ClientIP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub load_balancer: Option<LoadBalancerStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStatus {
    pub ingress: Vec<LoadBalancerIngress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerIngress {
    pub ip: Option<String>,
    pub hostname: Option<String>,
}

impl InfralinkScheduler {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            pods: HashMap::new(),
            deployments: HashMap::new(),
            services: HashMap::new(),
        }
    }

    /// Add a node to the cluster
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Schedule a pod to a suitable node
    pub async fn schedule_pod(&mut self, mut pod: Pod) -> Result<String, Box<dyn std::error::Error>> {
        let node = self.find_suitable_node(&pod)?;
        
        pod.node_id = Some(node.id.clone());
        pod.status.phase = PodPhase::Pending;
        
        // Add scheduling condition
        pod.status.conditions.push(PodCondition {
            condition_type: PodConditionType::PodScheduled,
            status: ConditionStatus::True,
            last_probe_time: Some(Utc::now()),
            last_transition_time: Utc::now(),
            reason: Some("Scheduled".to_string()),
            message: Some(format!("Successfully assigned to node {}", node.name)),
        });

        // Create containers on the scheduled node
        self.create_pod_containers(&mut pod, &node).await?;
        
        let pod_id = pod.id.clone();
        self.pods.insert(pod_id.clone(), pod);
        
        Ok(pod_id)
    }

    /// Find a suitable node for scheduling the pod
    fn find_suitable_node(&self, pod: &Pod) -> Result<&Node, Box<dyn std::error::Error>> {
        let mut candidate_nodes: Vec<&Node> = self.nodes.iter()
            .filter(|node| {
                // Filter by node status
                matches!(node.status, NodeStatus::Ready) &&
                // Check resource requirements
                self.node_has_resources(node, &pod.spec.resources) &&
                // Check node selector
                self.node_matches_selector(node, &pod.spec.node_selector) &&
                // Check taints and tolerations
                self.pod_tolerates_node_taints(pod, node)
            })
            .collect();

        if candidate_nodes.is_empty() {
            return Err("No suitable nodes found for scheduling".into());
        }

        // Apply affinity rules
        if let Some(affinity) = &pod.spec.affinity {
            candidate_nodes = self.apply_affinity_rules(candidate_nodes, affinity, pod);
        }

        // Use first-fit scheduling for now (can be improved with more sophisticated algorithms)
        candidate_nodes.first()
            .copied()
            .ok_or_else(|| "No nodes available after affinity filtering".into())
    }

    fn node_has_resources(&self, node: &Node, requirements: &ResourceRequirements) -> bool {
        let available_cpu = node.allocatable.cpu_cores - node.allocated.cpu_cores;
        let available_memory = node.allocatable.memory_mb - node.allocated.memory_mb;
        let available_storage = node.allocatable.storage_gb - node.allocated.storage_gb;

        available_cpu >= requirements.requests.cpu_cores &&
        available_memory >= requirements.requests.memory_mb &&
        available_storage >= requirements.requests.storage_gb
    }

    fn node_matches_selector(&self, node: &Node, selector: &HashMap<String, String>) -> bool {
        for (key, value) in selector {
            if node.labels.get(key) != Some(value) {
                return false;
            }
        }
        true
    }

    fn pod_tolerates_node_taints(&self, pod: &Pod, node: &Node) -> bool {
        for taint in &node.taints {
            if !pod.spec.tolerations.iter().any(|toleration| {
                self.toleration_matches_taint(toleration, taint)
            }) {
                return false;
            }
        }
        true
    }

    fn toleration_matches_taint(&self, toleration: &Toleration, taint: &Taint) -> bool {
        if toleration.key != taint.key {
            return false;
        }

        match toleration.operator {
            TolerationOperator::Exists => true,
            TolerationOperator::Equal => {
                toleration.value.as_ref() == taint.value.as_ref()
            }
        }
    }

    fn apply_affinity_rules<'a>(&self, nodes: Vec<&'a Node>, _affinity: &Affinity, _pod: &Pod) -> Vec<&'a Node> {
        // TODO: Implement affinity rules
        // For now, return nodes as-is
        nodes
    }

    /// Create containers for a pod on the specified node
    async fn create_pod_containers(&self, pod: &mut Pod, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        for container_spec in &pod.spec.containers {
            match &node.provider {
                CloudProvider::Local => {
                    self.create_local_container(container_spec, pod).await?;
                }
                CloudProvider::Fly { app_name } => {
                    self.create_fly_container(container_spec, pod, app_name).await?;
                }
                CloudProvider::Vultr { .. } | CloudProvider::Hetzner { .. } => {
                    // TODO: Implement for other cloud providers
                    return Err("Cloud provider not yet implemented for container creation".into());
                }
            }
        }

        pod.status.phase = PodPhase::Running;
        pod.status.start_time = Some(Utc::now());
        
        Ok(())
    }

    async fn create_local_container(&self, container_spec: &ContainerSpec, pod: &Pod) -> Result<(), Box<dyn std::error::Error>> {
        let request = CreateContainerRequest {
            name: Some(format!("{}-{}", pod.name, container_spec.name)),
            image: container_spec.image.clone(),
            command: container_spec.command.clone(),
            env: Some(container_spec.env.iter().map(|env| {
                format!("{}={}", env.name, env.value.as_ref().unwrap_or(&String::new()))
            }).collect()),
            ports: Some(container_spec.ports.iter().map(|port| {
                crate::providers::local::models::request::container::PortMapping {
                    host_port: port.container_port, // TODO: Implement proper port allocation
                    container_port: port.container_port,
                    protocol: port.protocol.clone(),
                }
            }).collect()),
            volumes: Some(container_spec.volume_mounts.iter().map(|mount| {
                crate::providers::local::models::request::container::VolumeMount {
                    source: mount.name.clone(), // TODO: Map to actual volume
                    destination: mount.mount_path.clone(),
                    mode: if mount.read_only { "ro".to_string() } else { "rw".to_string() },
                }
            }).collect()),
            labels: Some({
                let mut labels = HashMap::new();
                labels.insert("infralink.pod".to_string(), pod.id.clone());
                labels.insert("infralink.container".to_string(), container_spec.name.clone());
                labels
            }),
            restart_policy: Some("always".to_string()),
            working_dir: None,
            user: None,
            memory_limit: Some(container_spec.resources.limits.memory_mb as i64 * 1024 * 1024),
            cpu_limit: Some((container_spec.resources.limits.cpu_cores * 1_000_000_000.0) as i64),
            auto_remove: Some(false),
            detach: Some(true),
        };

        let container_id = LocalContainer::create(request).await?;
        LocalContainer::start(&container_id).await?;
        
        println!("Created local container {} for pod {}", container_id, pod.name);
        Ok(())
    }

    async fn create_fly_container(&self, container_spec: &ContainerSpec, pod: &Pod, app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let machine_config = MachineConfig {
            image: container_spec.image.clone(),
            guest: GuestConfig {
                cpu_kind: "shared".to_string(),
                cpus: container_spec.resources.requests.cpu_cores.ceil() as u32,
                memory_mb: container_spec.resources.requests.memory_mb as u32,
                gpu_kind: None,
            },
            env: Some(container_spec.env.iter().map(|env| {
                (env.name.clone(), env.value.as_ref().unwrap_or(&String::new()).clone())
            }).collect()),
            services: None, // TODO: Map container ports to services
            init: container_spec.command.as_ref().map(|cmd| {
                crate::providers::fly::models::request::machine::InitConfig {
                    exec: cmd.clone(),
                    entrypoint: None,
                    cmd: container_spec.args.clone(),
                }
            }),
            restart: Some(crate::providers::fly::models::request::machine::RestartConfig {
                policy: "always".to_string(),
            }),
            checks: None,
            mounts: None, // TODO: Map volume mounts
            auto_destroy: Some(false),
            metadata: Some({
                let mut metadata = HashMap::new();
                metadata.insert("infralink.pod".to_string(), pod.id.clone());
                metadata.insert("infralink.container".to_string(), container_spec.name.clone());
                metadata
            }),
        };

        let request = CreateMachineRequest {
            config: machine_config,
            name: Some(format!("{}-{}", pod.name, container_spec.name)),
            region: None, // Use default region
            skip_launch: Some(false),
            skip_service_registration: Some(false),
        };

        let machine = FlyMachine::create(app_name, request).await?;
        println!("Created Fly machine {} for pod {}", machine.id, pod.name);
        
        Ok(())
    }

    /// Create a deployment and manage its replica sets
    pub async fn create_deployment(&mut self, deployment: Deployment) -> Result<String, Box<dyn std::error::Error>> {
        let deployment_id = deployment.id.clone();
        
        // Create initial pods based on replica count
        for i in 0..deployment.spec.replicas {
            let pod = self.create_pod_from_template(&deployment, i).await?;
            self.schedule_pod(pod).await?;
        }

        self.deployments.insert(deployment_id.clone(), deployment);
        Ok(deployment_id)
    }

    async fn create_pod_from_template(&self, deployment: &Deployment, replica_index: u32) -> Result<Pod, Box<dyn std::error::Error>> {
        let pod_name = format!("{}-{}-{}", deployment.name, Uuid::new_v4().to_string()[..8].to_string(), replica_index);
        
        Ok(Pod {
            id: Uuid::new_v4().to_string(),
            name: pod_name,
            namespace: deployment.namespace.clone(),
            spec: deployment.spec.template.spec.clone(),
            status: PodStatus {
                phase: PodPhase::Pending,
                conditions: Vec::new(),
                container_statuses: Vec::new(),
                pod_ip: None,
                start_time: None,
            },
            node_id: None,
            created_at: Utc::now(),
            labels: deployment.spec.template.metadata.labels.clone(),
            annotations: deployment.spec.template.metadata.annotations.clone(),
        })
    }

    /// Create a service for load balancing and discovery
    pub fn create_service(&mut self, service: Service) -> Result<String, Box<dyn std::error::Error>> {
        let service_id = service.id.clone();
        self.services.insert(service_id.clone(), service);
        Ok(service_id)
    }

    /// Get pods matching a service selector
    pub fn get_service_endpoints(&self, service: &Service) -> Vec<&Pod> {
        self.pods.values()
            .filter(|pod| {
                // Check if pod matches service selector
                service.spec.selector.iter().all(|(key, value)| {
                    pod.labels.get(key) == Some(value)
                }) && matches!(pod.status.phase, PodPhase::Running)
            })
            .collect()
    }

    /// Update node resource allocation
    pub fn update_node_allocation(&mut self, node_id: &str, resources: &ResourceSpec) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.allocated.cpu_cores += resources.cpu_cores;
            node.allocated.memory_mb += resources.memory_mb;
            node.allocated.storage_gb += resources.storage_gb;
        }
    }

    /// Scale a deployment
    pub async fn scale_deployment(&mut self, deployment_id: &str, replicas: u32) -> Result<(), Box<dyn std::error::Error>> {
        let (current_replicas, deployment_clone) = {
            if let Some(deployment) = self.deployments.get_mut(deployment_id) {
                let current_replicas = deployment.spec.replicas;
                deployment.spec.replicas = replicas;
                (current_replicas, deployment.clone())
            } else {
                return Err("Deployment not found".into());
            }
        };

        if replicas > current_replicas {
            // Scale up: create new pods
            for i in current_replicas..replicas {
                let pod = self.create_pod_from_template(&deployment_clone, i).await?;
                self.schedule_pod(pod).await?;
            }
        } else if replicas < current_replicas {
            // Scale down: remove excess pods
            let pods_to_remove: Vec<String> = self.pods.values()
                .filter(|pod| {
                    deployment_clone.spec.selector.match_labels.iter().all(|(key, value)| {
                        pod.labels.get(key) == Some(value)
                    })
                })
                .take((current_replicas - replicas) as usize)
                .map(|pod| pod.id.clone())
                .collect();

            for pod_id in pods_to_remove {
                self.delete_pod(&pod_id).await?;
            }
        }

        Ok(())
    }

    /// Delete a pod and its containers
    pub async fn delete_pod(&mut self, pod_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pod) = self.pods.remove(pod_id) {
            if let Some(node_id) = &pod.node_id {
                if let Some(node) = self.nodes.iter().find(|n| n.id == *node_id) {
                    // Delete containers based on provider
                    match &node.provider {
                        CloudProvider::Local => {
                            // TODO: Get actual container IDs and delete them
                            println!("Deleting local containers for pod {}", pod.name);
                        }
                        CloudProvider::Fly { app_name: _ } => {
                            // TODO: Delete Fly machines
                            println!("Deleting Fly machines for pod {}", pod.name);
                        }
                        _ => {
                            println!("Provider not implemented for container deletion");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for InfralinkScheduler {
    fn default() -> Self {
        Self::new()
    }
}