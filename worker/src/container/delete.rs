use podman_api::Podman;
use podman_api::opts::ContainerDeleteOpts;

use std::env;
use std::path::PathBuf;

use tonic::Request;
use tonic::Response;
use tonic::Status;

mod podman {
	include!("../podman.rs");
}

use podman::{DeleteContainerRequest, DeleteContainerResponse};

#[tonic::async_trait]
pub trait ContainerDeleteService {
	async fn delete_container(
		&self,
		request: Request<DeleteContainerRequest>,
	) -> Result<Response<DeleteContainerResponse>, Status>;
}

fn get_socket_path() -> Result<PathBuf, std::io::Error> {
	let path = env::var("PODMAN_SOCKET_PATH").unwrap_or("/var/run/podman/podman.sock".to_string());
	Ok(PathBuf::from(path))
}

pub struct ContainerDeleteServiceImpl {};

#[tonic::async_trait]
impl ContainerDeleteService for ContainerDeleteServiceImpl {
	async fn delete_container(
		&self,
		request: Request<DeleteContainerRequest>,
	) -> Result<Response<DeleteContainerResponse>, Status> {
		let socket_path = get_socket_path()?;
		let client = Podman::unix(&socket_path);

		let container_id = request.into_inner().container_id;

		let delete_container_opts = ContainerDeleteOpts::builder()
        	.force()
        	.build();

		let container = client.containers().get(&container_id.to_owned());
		let response = containers().delete(&ContainerDeleteOpts::builder().volumes(true).build());

		let delete_container_response = match response.await {
			Ok(response) => {
				DeleteContainerResponse {
					message: format!("Container {} deleted", container_id),
				}
			}
			Err(err) => {
				println!("Could not delete container: {}", err);
				return Err(Status::internal("Failed to delete container"))
			}
		};
	}
}
