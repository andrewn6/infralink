use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, RemoveContainerOptions};
use reqwest::StatusCode;

use std::collections::HashMap;
use tokio::runtime::Runtime;
use tonic::{Request, Response, Status};

mod docker {
	include!("../docker.rs");
}

use docker::{StartContainerRequest, StopContainerRequest, DeleteContainerRequest, Container, Pod, CreatePodResponse};

#[tonic::async_trait]
pub trait DockerService {
	async fn create_container(
		&self,
		request: Request<Pod>,
	) -> Result<Response<CreatePodResponse>, Status>;

	async fn start_container(
		&self,
		request: Request<StartContainerRequest>,
	) -> Result<Response<()>, Status>;

	async fn stop_container(
		&self,
		request: Request<StopContainerRequest>,
	) -> Result<Response<()>, Status>;
	
	async fn delete_container(
		&self,
		request: Request<DeleteContainerRequest>,
	) -> Result <Response<()>, Status>;
}

pub struct MyDockerService {}

#[tonic::async_trait]
impl DockerService for MyDockerService {
	async fn create_container(
		&self,
		request: Request<Pod>,
	) -> Result<Response<CreatePodResponse>, Status> {	
		let request = request.into_inner();

		let docker = Docker::connect_with_local_defaults().unwrap();
		let mut exposed_ports = HashMap::new();

		for port in request.containers.iter().flat_map(|c| c.ports.iter()) {
			exposed_ports.insert(port.clone(), HashMap::new());
		}

		let config = Config {
			image: Some(request.containers[0].image.clone()),
			env: Some(request
				.containers[0]
				.env
				.iter()
				.map(|(k, v)| format!("{}={}", k, v))
				.collect::<Vec<_>>()),
			cmd: Some(request.containers[0].commands),
			exposed_ports: Some(exposed_ports),
			..Default::default()
		};

		let options = Some(CreateContainerOptions { 
			name: request.containers[0].name.clone(),
			platform: Some("linux/amd64".to_owned()),
		});

		match docker.create_container(options, config).await {
        	Ok(container) => {
            	let message = format!("Created container with ID: {}", container.id);
            	let response = CreatePodResponse { message };
            	Ok(Response::new(response))
        	}
        	Err(err) => {
            	eprintln!("Error creating container: {:?}", err);
            	Err(Status::internal("Failed to create container"))
        	}
    }
	}

	async fn start_container(
		&self,
		request: Request<StartContainerRequest>,
	) -> Result<Response<()>, Status> {
		unimplemented!()
	}

	async fn stop_container(
		&self,
		request: Request<StopContainerRequest>,
	) -> Result<Response<()>, Status> {
		unimplemented!()
	}

	async fn delete_container(
		&self,
		request: Request<DeleteContainerRequest>,
	) -> Result <Response<()>, Status> {
		unimplemented!()
	}
		
}