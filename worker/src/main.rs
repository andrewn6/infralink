use std::collections::HashMap;
use std::time::Duration;

use podman_api::api::Container;
use podman_api::models::ContainerStats;
use podman_api::opts::ContainerStatsOpts;
use podman_api::opts::{ContainerListOpts};
use podman_api::{Podman};

use tonic::{transport::Server, Request, Response, Status};
// use tonic::{Request, Response, Status};

// Load in gRPC service definitions
// use proto_memory::memory_service_server::{MemoryService, MemoryServiceServer};
// use proto_memory::MemoryMetadata;

// use proto_compute::compute_service_server::{ComputeService, ComputeServiceServer};
// use proto_compute::ComputeMetadata;

// use proto_network::network_service_server::{NetworkService, NetworkServiceServer};
// use proto_network::NetworkMetadata;

// use proto_storage::storage_service_server::{StorageService, StorageServiceServer};
// use proto_storage::StorageMetadata;

// use std::sync::Arc;
use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

mod hello_world {
    include!("helloworld.rs");
}

mod proto_compute {
	include!("compute.rs");
}

mod proto_memory {
	include!("memory.rs");

	pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
		tonic::include_file_descriptor_set!("greeter_descriptor");
}

mod proto_storage {
	include!("storage.rs");
}

mod proto_network {
	include!("network.rs");
}

#[derive(Default)]
pub struct ComputeServiceImpl {}

#[derive(Default)]
pub struct MemoryServiceImpl {}

#[derive(Default)]
pub struct StorageServiceImpl {}

#[derive(Default)]
pub struct NetworkServiceImpl {}

#[derive(Default)]
pub struct MyGreeter {}

#[derive(Default)]
struct ContainerStatsImpl {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tonic::async_trait]
trait GetStats {
	async fn get_stats(&self, request: Request<()>) -> Result<Response<HashMap<String, String>>, Status>;
}

#[tonic::async_trait]
impl GetStats for ContainerStats {
    async fn get_stats(&self, request: Request<()>) -> Result<Response<HashMap<String, String>>, Status> {
        let mut stats_result: HashMap<String, String> = HashMap::new();

        let podman = Podman::unix("unix:///var/run/podman/podman.sock");
        let container_stats_opts = podman_api::opts::ContainerStatsOpts::default();
		let container_list_opts = podman_api::opts::ContainerListOpts::default();
        let containers = podman.containers().list(&container_list_opts).await;

        for container in containers {
			let stats = podman.containers().stats(&container_stats_opts).await;
			for (key, value) in stats.into_iter() {
				let value_str = value.unwrap_or_default(stats).to_string();
				stats_result.insert(key, value_str);
			}
		}
		Ok(Response::new(stats_result))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

	// Initialize the memory, compute, storage, and network measurement services
	// let memory_service = MemoryServiceImpl::default();
	// let compute_service = ComputeServiceImpl::default();
	// let storage_service = StorageServiceImpl::default();
	// Change the param to true if we want to meter incoming bandwidth
	// let network_service = NetworkServiceImpl::default();

	// Create the gRPC servers for each service
	// let memory_server = MemoryServiceServer::new(memory_service);
	// let compute_server = ComputeServiceServer::new(compute_service);
	// let network_server = NetworkServiceServer::new(network_service);
	// let storage_server = StorageServiceServer::new(storage_service);

	let reflection_service = tonic_reflection::server::Builder::configure()
		.register_encoded_file_descriptor_set(proto_memory::FILE_DESCRIPTOR_SET)
		.build()
		.unwrap();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
		.add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}