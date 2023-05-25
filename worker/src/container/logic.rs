use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions};
use bollard::Docker;

use std::collections::HashMap;
use tonic::{Request, Response, Status};

mod docker {
	include!("../docker.rs");
}

use docker::{
	CreatePodResponse, DeleteContainerRequest, Pod, StartContainerRequest, StopContainerRequest,
};

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
	) -> Result<Response<()>, Status>;
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
			env: Some(
				request.containers[0]
					.env
					.iter()
					.map(|(k, v)| format!("{}={}", k, v))
					.collect::<Vec<_>>(),
			),
			cmd: Some(request.containers[0].commands.clone()),
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
		let request = request.into_inner();

		let docker = Docker::connect_with_local_defaults().unwrap();

		match docker
			.start_container::<String>(&request.container_id, None)
			.await
		{
			Ok(_) => Ok(Response::new(())),

			Err(err) => {
				eprintln!("Error starting container: {:?}", err);
				Err(Status::internal("Failed to start container"))
			}
		}
	}

	async fn stop_container(
		&self,
		request: Request<StopContainerRequest>,
	) -> Result<Response<()>, Status> {
		let request = request.into_inner();

		let docker = Docker::connect_with_local_defaults().unwrap();

		match docker.stop_container(&request.name, None).await {
			Ok(_) => Ok(Response::new(())),
			Err(err) => {
				eprintln!("Error stopping container: {:?}", err);
				Err(Status::internal("Failed to stop container"))
			}
		}
	}

	async fn delete_container(
		&self,
		request: Request<DeleteContainerRequest>,
	) -> Result<Response<()>, Status> {
		let request = request.into_inner();

		let docker = Docker::connect_with_local_defaults().unwrap();

		let options = Some(RemoveContainerOptions {
			force: true,
			v: true,
			..Default::default()
		});

		match docker
			.remove_container(&request.container_id, options)
			.await
		{
			Ok(_) => Ok(Response::new(())),
			Err(err) => {
				eprintln!("Error deleting container: {:?}", err);
				Err(Status::internal("Failed to delete container"))
			}
		}
	}
}
