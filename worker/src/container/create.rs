use std::convert::TryInto;
use std::env;
use tonic::transport::Channel;
use tokio::runtime::Runtime;
use podman_api::Podman;

pub mod podman {
    tonic::include_proto!("../podman");
}
use podman::{PodmanClient, Container, ContainerListOpts, CreatePodResponse, Pod, StartContainerRequest, StartContainerResponse};

#[derive(Default)]
pub struct ContainerCreateService {}

#[tonic::async_trait]
impl Container for ContainerCreateService {
    async fn create_pod(
        &self,
        request: Request<Pod>,
    ) -> Result<Response<CreatePodResponse>, Status> {
        let mut client = Podman::unix("unix:///var/run/podman/podman.sock");

        let response = client.create_pod(&request.into_inner()).await?.into_inner();
        println!("Pod Response: {:?}", response);

        Ok(Response::new(response))
    }

    async fn start_container(
        &self,
        request: Request<StartContainerRequest>,
    ) -> Result<Response<StartContainerResponse>, Status> {
        let mut client = Podman::unix("unix:///var/run/podman/podman.sock");

        let response = client.start_container(&request.into_inner()).await?.into_inner();
        println!("Start Container Response: {:?}", response);

        Ok(Response::new(response))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Podman::unix("unix:///var/run/podman/podman.sock");

    let mut container = Container::default();
    container.image = "alpine:latest".to_string();
    container.name = "alpine".to_string();
    container.commands = vec!["/bin/bash".to_string(), "-c".to_string(), "echo 'Hello, World'".to_string()];

    let mut pod = Pod::default();
    pod.containers = vec![container];

    let response = client.create_pod(&pod).await?.into_inner();
    println!("Pod Response: {:?}", response);

    let request = StartContainerRequest {
        name: "container-1".to_string(),
    };

    let respones = client.start_container(request).await?.into_inner();
    println!("Start Container Response: {:?}", respones);

    Ok(())
}