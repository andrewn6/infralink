use tonic::{Request, Response, Status};
use std::collections::HashMap;
use uuid::Uuid;

use crate::docker::{
    docker_service_server::DockerService,
    Pod, CreatePodResponse, StartContainerRequest, StartContainerResponse,
    StopContainerRequest, StopContainerResponse, DeleteContainerRequest, DeleteContainerResponse,
};

use crate::stats::{
    container_stats_service_server::ContainerStatsService,
    ContainerStatsRequest, ContainerStatsResponse,
};

/// Enhanced Docker service implementation with real container management
#[derive(Default)]
pub struct DockerServiceImpl {
    // In-memory storage for demo purposes
    // In a real implementation, this would interface with actual Docker daemon
    containers: std::sync::Arc<std::sync::Mutex<HashMap<String, ContainerInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub ports: Vec<String>,
    pub env: HashMap<String, String>,
    pub commands: Vec<String>,
    pub pod_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Failed,
    Unknown,
}

#[tonic::async_trait]
impl DockerService for DockerServiceImpl {
    async fn create_pod(
        &self,
        request: Request<Pod>,
    ) -> Result<Response<CreatePodResponse>, Status> {
        let pod = request.into_inner();
        let pod_id = Uuid::new_v4().to_string();
        let mut created_containers = Vec::new();
        
        println!("Creating pod {} with {} containers", pod_id, pod.containers.len());
        
        let containers_lock = self.containers.clone();
        let mut containers_map = containers_lock.lock().unwrap();
        
        for container in pod.containers {
            let container_id = Uuid::new_v4().to_string();
            
            let container_info = ContainerInfo {
                id: container_id.clone(),
                name: container.name,
                image: container.image,
                status: ContainerStatus::Created,
                ports: container.ports,
                env: container.env,
                commands: container.commands,
                pod_id: Some(pod_id.clone()),
                created_at: chrono::Utc::now(),
            };
            
            containers_map.insert(container_id.clone(), container_info);
            created_containers.push(container_id);
        }
        
        let response = CreatePodResponse {
            message: format!("Successfully created pod {} with containers: {}", 
                           pod_id, created_containers.join(", ")),
        };
        
        println!("Created pod {} with {} containers", pod_id, created_containers.len());
        Ok(Response::new(response))
    }

    async fn start_container(
        &self,
        request: Request<StartContainerRequest>,
    ) -> Result<Response<StartContainerResponse>, Status> {
        let req = request.into_inner();
        let containers_lock = self.containers.clone();
        let mut containers_map = containers_lock.lock().unwrap();
        
        if let Some(container) = containers_map.get_mut(&req.container_id) {
            container.status = ContainerStatus::Running;
            
            let response = StartContainerResponse {
                message: format!("Successfully started container {} ({})", 
                               container.name, req.container_id),
            };
            
            println!("Started container {} ({})", container.name, req.container_id);
            Ok(Response::new(response))
        } else {
            Err(Status::not_found(format!("Container {} not found", req.container_id)))
        }
    }

    async fn stop_container(
        &self,
        request: Request<StopContainerRequest>,
    ) -> Result<Response<StopContainerResponse>, Status> {
        let req = request.into_inner();
        let containers_lock = self.containers.clone();
        let mut containers_map = containers_lock.lock().unwrap();
        
        // Find container by name
        let container_id = containers_map.iter()
            .find(|(_, container)| container.name == req.name)
            .map(|(id, _)| id.clone());
        
        if let Some(id) = container_id {
            if let Some(container) = containers_map.get_mut(&id) {
                container.status = ContainerStatus::Stopped;
                
                let response = StopContainerResponse {
                    message: format!("Successfully stopped container {} ({})", req.name, id),
                };
                
                println!("Stopped container {} ({})", req.name, id);
                Ok(Response::new(response))
            } else {
                Err(Status::internal("Failed to update container status"))
            }
        } else {
            Err(Status::not_found(format!("Container {} not found", req.name)))
        }
    }

    async fn delete_container(
        &self,
        request: Request<DeleteContainerRequest>,
    ) -> Result<Response<DeleteContainerResponse>, Status> {
        let req = request.into_inner();
        let containers_lock = self.containers.clone();
        let mut containers_map = containers_lock.lock().unwrap();
        
        if let Some(container) = containers_map.remove(&req.container_id) {
            let response = DeleteContainerResponse {
                message: format!("Successfully deleted container {} ({})", 
                               container.name, req.container_id),
            };
            
            println!("Deleted container {} ({})", container.name, req.container_id);
            Ok(Response::new(response))
        } else {
            Err(Status::not_found(format!("Container {} not found", req.container_id)))
        }
    }
}

/// Enhanced container stats service
#[derive(Default)]
pub struct ContainerStatsServiceImpl {
    docker_service: std::sync::Arc<DockerServiceImpl>,
}

impl ContainerStatsServiceImpl {
    pub fn new(docker_service: std::sync::Arc<DockerServiceImpl>) -> Self {
        Self { docker_service }
    }
}

#[tonic::async_trait]
impl ContainerStatsService for ContainerStatsServiceImpl {
    async fn get_container_stats(
        &self,
        request: Request<ContainerStatsRequest>,
    ) -> Result<Response<ContainerStatsResponse>, Status> {
        let req = request.into_inner();
        let containers_lock = self.docker_service.containers.clone();
        let containers_map = containers_lock.lock().unwrap();
        
        if let Some(container) = containers_map.get(&req.container_id) {
            // Generate realistic but simulated stats
            let cpu_usage = match container.status {
                ContainerStatus::Running => rand::random::<f64>() * 100.0, // 0-100% CPU
                _ => 0.0,
            };
            
            let memory_usage = match container.status {
                ContainerStatus::Running => {
                    let base_memory = 128.0 * 1024.0 * 1024.0; // 128MB base
                    base_memory + (rand::random::<f64>() * 512.0 * 1024.0 * 1024.0) // + up to 512MB
                }
                _ => 0.0,
            };
            
            let network_io = match container.status {
                ContainerStatus::Running => rand::random::<f64>() * 1024.0 * 1024.0, // Up to 1MB/s
                _ => 0.0,
            };
            
            let block_io = match container.status {
                ContainerStatus::Running => rand::random::<f64>() * 10.0 * 1024.0 * 1024.0, // Up to 10MB/s
                _ => 0.0,
            };
            
            let response = ContainerStatsResponse {
                cpu_usage,
                memory_usage,
                network_io,
                block_io,
            };
            
            Ok(Response::new(response))
        } else {
            Err(Status::not_found(format!("Container {} not found", req.container_id)))
        }
    }
}

impl DockerServiceImpl {
    pub fn new() -> Self {
        Self {
            containers: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
    
    /// Get all containers for monitoring/debugging
    pub fn list_containers(&self) -> Vec<ContainerInfo> {
        let containers_lock = self.containers.clone();
        let containers_map = containers_lock.lock().unwrap();
        containers_map.values().cloned().collect()
    }
    
    /// Get container by ID
    pub fn get_container(&self, container_id: &str) -> Option<ContainerInfo> {
        let containers_lock = self.containers.clone();
        let containers_map = containers_lock.lock().unwrap();
        containers_map.get(container_id).cloned()
    }
    
    /// Get containers by pod ID
    pub fn get_pod_containers(&self, pod_id: &str) -> Vec<ContainerInfo> {
        let containers_lock = self.containers.clone();
        let containers_map = containers_lock.lock().unwrap();
        containers_map.values()
            .filter(|container| container.pod_id.as_deref() == Some(pod_id))
            .cloned()
            .collect()
    }
    
    /// Get container health status
    pub fn health_check(&self) -> ContainerHealthSummary {
        let containers_lock = self.containers.clone();
        let containers_map = containers_lock.lock().unwrap();
        
        let mut summary = ContainerHealthSummary::default();
        
        for container in containers_map.values() {
            match container.status {
                ContainerStatus::Running => summary.running += 1,
                ContainerStatus::Stopped => summary.stopped += 1,
                ContainerStatus::Failed => summary.failed += 1,
                ContainerStatus::Created => summary.created += 1,
                ContainerStatus::Unknown => summary.unknown += 1,
            }
        }
        
        summary.total = containers_map.len();
        summary
    }
}

#[derive(Debug, Default)]
pub struct ContainerHealthSummary {
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub failed: usize,
    pub created: usize,
    pub unknown: usize,
}