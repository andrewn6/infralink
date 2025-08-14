use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::providers::local::docker_client::{
    DockerClient, ContainerSpec, ContainerInfo, ContainerState,
    PortMapping as DockerPortMapping, VolumeMount as DockerVolumeMount, 
    ResourceLimits, ContainerRestartPolicy, DockerError, ContainerStats,
    VolumeMountType,
};

lazy_static::lazy_static! {
    static ref DOCKER_CLIENT: Arc<Mutex<Option<Arc<DockerClient>>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: Vec<PortMapping>,
    pub env: Vec<String>,
    pub labels: HashMap<String, String>,
    pub created: String,
    pub command: Vec<String>,
    pub mounts: Vec<VolumeMount>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String, // "tcp" or "udp"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeMount {
    pub source: String,
    pub destination: String,
    pub mode: String, // "rw" or "ro"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateContainerRequest {
    pub name: Option<String>,
    pub image: String,
    pub command: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volumes: Option<Vec<VolumeMount>>,
    pub labels: Option<HashMap<String, String>>,
    pub restart_policy: Option<String>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
    pub memory_limit: Option<i64>,
    pub cpu_limit: Option<i64>,
    pub auto_remove: Option<bool>,
    pub detach: Option<bool>,
}

/// Get or initialize the Docker client
async fn get_docker_client() -> Result<Arc<DockerClient>, ContainerError> {
    let mut client_guard = DOCKER_CLIENT.lock().await;
    
    if client_guard.is_none() {
        let docker_client = DockerClient::new().await
            .map_err(|e| ContainerError::InitializationFailed(e.to_string()))?;
        *client_guard = Some(Arc::new(docker_client));
    }
    
    Ok(client_guard.as_ref().unwrap().clone())
}

impl LocalContainer {
    /// Create a new container using real Docker API
    pub async fn create(
        request: CreateContainerRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        // Convert request to Docker spec
        let spec = convert_to_docker_spec(request)?;
        
        let container_id = docker_client.create_container(&spec).await
            .map_err(|e| ContainerError::CreationFailed(e.to_string()))?;
        
        println!("Created local container {} with image {}", container_id, spec.image);
        Ok(container_id)
    }

    /// Start a container using real Docker API
    pub async fn start(container_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        docker_client.start_container(container_id).await
            .map_err(|e| ContainerError::OperationFailed(e.to_string()))?;
        
        println!("Started local container {}", container_id);
        Ok(())
    }

    /// Stop a container using real Docker API
    pub async fn stop(container_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        docker_client.stop_container(container_id, Some(10)).await
            .map_err(|e| ContainerError::OperationFailed(e.to_string()))?;
        
        println!("Stopped local container {}", container_id);
        Ok(())
    }

    /// Remove a container using real Docker API
    pub async fn remove(container_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        docker_client.remove_container(container_id, true).await
            .map_err(|e| ContainerError::OperationFailed(e.to_string()))?;
        
        println!("Removed local container {}", container_id);
        Ok(())
    }

    /// List containers using real Docker API
    pub async fn list() -> Result<Vec<LocalContainer>, Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        let container_infos = docker_client.list_containers(true).await
            .map_err(|e| ContainerError::OperationFailed(e.to_string()))?;
        
        let containers = container_infos.into_iter()
            .map(convert_from_docker_info)
            .collect();
        
        println!("Listed {} local containers", containers.len());
        Ok(containers)
    }

    /// Inspect a container using real Docker API
    pub async fn inspect(container_id: &str) -> Result<LocalContainer, Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        let info = docker_client.inspect_container(container_id).await
            .map_err(|e| match e {
                DockerError::ContainerError(msg) if msg.contains("No such container") => {
                    ContainerError::NotFound(container_id.to_string())
                },
                _ => ContainerError::OperationFailed(e.to_string()),
            })?;
        
        let container = convert_from_docker_info(info);
        println!("Inspected local container {}", container_id);
        Ok(container)
    }

    /// Get container logs using real Docker API
    pub async fn logs(
        container_id: &str,
        follow: bool,
        tail: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        let lines = if follow {
            None // Follow mode doesn't use line limit
        } else {
            tail.and_then(|t| t.parse().ok())
        };
        
        let logs = docker_client.get_container_logs(container_id, lines).await
            .map_err(|e| ContainerError::OperationFailed(e.to_string()))?;
        
        println!("Retrieved logs for local container {} (follow: {}, tail: {:?})", 
                 container_id, follow, tail);
        Ok(logs)
    }

    /// Get container stats using real Docker API
    pub async fn stats(container_id: &str) -> Result<ContainerStats, Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        let stats = docker_client.get_container_stats(container_id).await
            .map_err(|e| ContainerError::OperationFailed(e.to_string()))?;
        
        Ok(stats)
    }

    /// Execute command in container using real Docker API
    pub async fn exec(
        container_id: &str, 
        cmd: Vec<String>
    ) -> Result<String, Box<dyn std::error::Error>> {
        let docker_client = get_docker_client().await?;
        
        let output = docker_client.exec_in_container(container_id, cmd.clone()).await
            .map_err(|e| ContainerError::OperationFailed(e.to_string()))?;
        
        println!("Executed command {:?} in container {}", cmd, container_id);
        Ok(output)
    }
}

/// Convert CreateContainerRequest to Docker ContainerSpec
fn convert_to_docker_spec(request: CreateContainerRequest) -> Result<ContainerSpec, ContainerError> {
    let name = request.name.unwrap_or_else(|| {
        format!("infralink-{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
    });

    // Convert environment variables
    let env = request.env.unwrap_or_default().into_iter()
        .filter_map(|e| {
            if let Some(eq_pos) = e.find('=') {
                let key = e[..eq_pos].to_string();
                let value = e[eq_pos + 1..].to_string();
                Some((key, value))
            } else {
                None
            }
        })
        .collect();

    // Convert port mappings
    let ports = request.ports.unwrap_or_default().into_iter()
        .map(|p| DockerPortMapping {
            container_port: p.container_port,
            host_port: Some(p.host_port),
            protocol: p.protocol,
            host_ip: None,
        })
        .collect();

    // Convert volume mounts
    let volumes = request.volumes.unwrap_or_default().into_iter()
        .map(|v| DockerVolumeMount {
            source: v.source,
            target: v.destination,
            read_only: v.mode == "ro",
            mount_type: VolumeMountType::Bind,
        })
        .collect();

    // Convert restart policy
    let restart_policy = match request.restart_policy.as_deref() {
        Some("always") => ContainerRestartPolicy::Always,
        Some("unless-stopped") => ContainerRestartPolicy::UnlessStopped,
        Some("on-failure") => ContainerRestartPolicy::OnFailure { max_retry_count: None },
        _ => ContainerRestartPolicy::No,
    };

    // Build labels
    let mut labels = request.labels.unwrap_or_default();
    labels.insert("infralink.managed".to_string(), "true".to_string());
    labels.insert("infralink.created".to_string(), chrono::Utc::now().to_rfc3339());

    Ok(ContainerSpec {
        name,
        image: request.image,
        command: request.command.unwrap_or_default(),
        args: vec![],
        env,
        ports,
        volumes,
        resources: ResourceLimits {
            memory: request.memory_limit,
            cpu_quota: request.cpu_limit,
            cpu_period: None,
            cpu_shares: None,
        },
        restart_policy,
        network_mode: "infralink-network".to_string(),
        labels,
        working_dir: request.working_dir,
        user: request.user,
    })
}

/// Convert Docker ContainerInfo to LocalContainer
fn convert_from_docker_info(info: ContainerInfo) -> LocalContainer {
    let state = match info.state {
        ContainerState::Created => "created",
        ContainerState::Running => "running",
        ContainerState::Paused => "paused",
        ContainerState::Restarting => "restarting",
        ContainerState::Removing => "removing",
        ContainerState::Exited => "exited",
        ContainerState::Dead => "dead",
    }.to_string();

    let ports = info.ports.into_iter()
        .filter_map(|p| p.host_port.map(|hp| PortMapping {
            host_port: hp,
            container_port: p.container_port,
            protocol: p.protocol,
        }))
        .collect();

    let mounts = info.mounts.into_iter()
        .map(|m| VolumeMount {
            source: m.source,
            destination: m.target,
            mode: if m.read_only { "ro" } else { "rw" }.to_string(),
        })
        .collect();

    LocalContainer {
        id: info.id,
        name: info.name,
        image: info.image,
        state,
        status: info.status,
        ports,
        env: vec![], // Docker doesn't easily expose env vars in list
        labels: info.labels,
        created: info.created.to_rfc3339(),
        command: vec![], // Would need to parse from Config
        mounts,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("Container manager initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Container creation failed: {0}")]
    CreationFailed(String),
    #[error("Container operation failed: {0}")]
    OperationFailed(String),
    #[error("Container not found: {0}")]
    NotFound(String),
}