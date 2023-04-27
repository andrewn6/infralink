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
pub mod resource;

mod proto_memory {
	include!("memory.rs");

	pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
		tonic::include_file_descriptor_set!("greeter_descriptor");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse().unwrap();

	let reflection_service = tonic_reflection::server::Builder::configure()
		.register_encoded_file_descriptor_set(proto_memory::FILE_DESCRIPTOR_SET)
		.build()
		.unwrap();

	println!("gRPC server listening on {}", addr);

	Server::builder()
		.add_service(reflection_service)
		.serve(addr)
		.await?;

	Ok(())
}
