use tonic::transport::Server;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse().unwrap();

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

	println!("gRPC server listening on {}", addr);

	Server::builder()
		// .add_service(memory_server)
		// .add_service(compute_server)
		// .add_service(storage_server)
		// .add_service(network_server)
		.add_service(reflection_service)
		.serve(addr)
		.await?;

	Ok(())
}
