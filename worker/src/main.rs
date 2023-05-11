use std::collections::HashMap;
use std::time::Duration;

use podman_api::models::{ContainerStats, ContainerStats200Response};
use podman_api::Podman;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
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

mod container {
<<<<<<< HEAD
    pub mod delete;
    pub mod create;
}

use container::delete;

use container::create;
=======
	pub mod create;
}

// use container::create::ContainerCreateService;
>>>>>>> c819a0ea3cbdd9772220e50c98036f468d1974f5

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
	async fn get_stats(
		&self,
		request: Request<()>,
	) -> Result<Response<HashMap<String, String>>, Status>;
}

#[tonic::async_trait]
impl GetStats for ContainerStats {
	async fn get_stats(
		&self,
		request: Request<()>,
	) -> Result<Response<HashMap<String, String>>, Status> {
		let mut stats_result: HashMap<String, String> = HashMap::new();

		// Change this accordingle
		let podman = Podman::unix("unix:///var/run/podman/podman.sock");
		let container_stats_opts = podman_api::opts::ContainerStatsOpts::default();
		let container_list_opts = podman_api::opts::ContainerListOpts::default();
		let containers = podman.containers().list(&container_list_opts).await;

		for container in containers {
			let stats: ContainerStats200Response = podman
				.containers()
				.stats(&container_stats_opts)
				.await
				.unwrap();
			for (key, value) in stats.as_object().unwrap() {
				let value_str = value.to_string();
				stats_result.insert(key.to_string(), value_str);
			}
		}
		
		Ok(Response::new(stats_result))
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse().unwrap();
	let greeter = MyGreeter::default();

	let reflection_service = tonic_reflection::server::Builder::configure()
		.register_encoded_file_descriptor_set(proto_memory::FILE_DESCRIPTOR_SET)
		.build()
		.unwrap();

	println!("GreeterServer listening on {}", addr);

	Server::builder()
		.add_service(GreeterServer::new(greeter))
		.add_service(reflection_service)
		// .add_service(<dyn ContainerCreateService>::std::default())
		.serve(addr)
		.await?;

	Ok(())
}
