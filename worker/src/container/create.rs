use std::convert::TryInto;
use std::env;
use tonic::transport::Channel;
use tokio::runtime::Runtime;

// Podman rust bindings
use podman_api::Podman;
use podman_api::models::{CreateContainerResponse}
use podman_api::models::PortMapping;
use podman_api::opts::ContainerCreateOpts

pub mod podman {
    tonic::include_proto!("../podman");
}

// Imports from generated protobuf to Rust.
use podman::{PodmanClient, Container, CreatePodResponse, Pod, StartContainerRequest, StartContainerResponse};

fn get_socket_path() -> Result<PathBuf, std::io::Error> {
    let path = env::var("PODMAN_SOCKET_PATH").unwrap_or("/var/run/podman/podman.sock".to_string());
    Ok(PathBuf::from(path))
}

#[derive(Default)]
pub struct ContainerCreateService {}

impl Container for ContainerCreateService {
    async fn create_pod(
        &self,
        request: Request<Pod>,
    ) -> Result<Response<CreatePodResponse>, Status> {
        let socket_path = get_socket_path()?;
        let mut client = Podman::unix(&socket_path);

        let create_pod_request = request.into_inner();

        let container = create_pod_request.containers.first().ok_or_else(|| {
            Status::invalid_argument("One container must be specified")
        })?;

        let image = container.image.clone();
        let name = container.name.clone();
        let commands = container.commands.clone();
        let ports = container.port.clone();
        let env = container.env.clone();

        let create_container_info = ContainerCreateInfo {
            image: Some(image),
            name: Some(name),
            cmd: Some(commands),
            env: HashMap::new(),
            ports: ports
                .iter()
                .map(|port_str| {
                    let port = port_str.parse().map_err(|_| {
                        Status::invalid_argument(format!("Invalid port: {}", port_str))
                    })?;
                    Ok(PortMapping {
                        host_ip: None,
                        host_port: Some(port),
                        container_port: Some(port),
                        protocol: Some("tcp".to_owned()),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            ..Default::default()
        };

        let create_container_opts = ContainerCreateOpts::builder()
            .create_info(create_container_info)
            .build();

        let response = client.containers().create(&create_container_opts).await?;
        let create_container_response: CreateContainerResponse = response.into_inner();

        let create_pod_response = CreatePodResponse {
            containers: vec![create_container_response],
        };

        Ok(Response::new(create_pod_response))
    };

    async fn start_container(
        &self,
        request: Request<StartContainerRequest>,
    ) -> Result<Response<StartContainerResponse>, Status> {
        socket_path = get_socket_path()?;
        let mut client = Podman::unix(&socket_path);

        let container_id = request
            .into_inner()
            .name
            .ok_or_else(|| Status::invalid_argument("Container name must be specified"))?;
        let response = client.containers.get(&container_id).start(None).await?;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
}
