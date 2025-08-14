use tonic::transport::Server;
use tonic::{Request, Response, Status};

// use std::sync::Arc;
use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

pub mod container;

pub mod container_service;
use container_service::{DockerServiceImpl, ContainerStatsServiceImpl};

pub mod stats {
	include!("stats.rs");
}

pub mod docker {
	include!("docker.rs");
}

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

//#[derive(Default)]
//pub struct MyDockerService {}

#[derive(Default)]
pub struct ComputeServiceImpl {}

#[tonic::async_trait]
impl proto_compute::compute_service_server::ComputeService for ComputeServiceImpl {
    async fn get_compute_metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto_compute::ComputeMetadata>, Status> {
        // Get system CPU information
        let num_cores = num_cpus::get() as u64;
        
        // Simulate CPU information - in reality this would come from system monitoring
        let mut cpus = Vec::new();
        for _i in 0..num_cores {
            cpus.push(proto_compute::Cpu {
                frequency: 2400, // 2.4 GHz base frequency
                load: rand::random::<f32>() * 100.0, // Random load percentage
            });
        }
        
        let metadata = proto_compute::ComputeMetadata {
            num_cores,
            cpus,
        };
        
        Ok(Response::new(metadata))
    }
}

#[derive(Default)]
pub struct MemoryServiceImpl {}

#[tonic::async_trait]
impl proto_memory::memory_service_server::MemoryService for MemoryServiceImpl {
    async fn get_memory_metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto_memory::MemoryMetadata>, Status> {
        // Get system memory information
        // In a real implementation, this would use sysinfo or similar
        let total_memory = 8 * 1024 * 1024 * 1024; // 8GB in bytes
        let used_memory = (total_memory as f64 * (rand::random::<f64>() * 0.8)) as u64; // Random usage up to 80%
        let free_memory = total_memory - used_memory;
        
        let primary_memory = proto_memory::Memory {
            total: total_memory,
            used: used_memory,
            free: free_memory,
        };
        
        // Simulate swap memory
        let swap_total = 2 * 1024 * 1024 * 1024; // 2GB swap
        let swap_used = (swap_total as f64 * (rand::random::<f64>() * 0.3)) as u64; // Random swap usage up to 30%
        let swap_free = swap_total - swap_used;
        
        let swap_memory = proto_memory::Memory {
            total: swap_total,
            used: swap_used,
            free: swap_free,
        };
        
        let metadata = proto_memory::MemoryMetadata {
            primary: Some(primary_memory),
            swap: Some(swap_memory),
        };
        
        Ok(Response::new(metadata))
    }
}

#[derive(Default)]
pub struct StorageServiceImpl {}

#[tonic::async_trait]
impl proto_storage::storage_service_server::StorageService for StorageServiceImpl {
    async fn get_storage_metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto_storage::StorageMetadata>, Status> {
        // Simulate primary storage (root volume)
        let primary_total = 500 * 1024 * 1024 * 1024; // 500GB
        let primary_used = (primary_total as f64 * (rand::random::<f64>() * 0.7)) as u64; // Random usage up to 70%
        let primary_free = primary_total - primary_used;
        
        let primary_volume = proto_storage::Volume {
            total: primary_total,
            used: primary_used,
            free: primary_free,
        };
        
        // Simulate additional volumes
        let mut volumes = Vec::new();
        
        // Data volume
        let data_total = 1024 * 1024 * 1024 * 1024; // 1TB
        let data_used = (data_total as f64 * (rand::random::<f64>() * 0.5)) as u64;
        let data_free = data_total - data_used;
        
        volumes.push(proto_storage::Volume {
            total: data_total,
            used: data_used,
            free: data_free,
        });
        
        let metadata = proto_storage::StorageMetadata {
            primary: Some(primary_volume),
            volumes,
        };
        
        Ok(Response::new(metadata))
    }
}

#[derive(Default)]
pub struct NetworkServiceImpl {}

#[tonic::async_trait]
impl proto_network::network_service_server::NetworkService for NetworkServiceImpl {
    async fn get_network_metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto_network::NetworkMetadata>, Status> {
        // Simulate network statistics
        let total_outbound = (rand::random::<u64>() % 1000000000) + 1000000; // Random bytes transferred
        let total_inbound = (rand::random::<u64>() % 1000000000) + 1000000;
        let avg_outbound_per_sec = rand::random::<u64>() % 1000000; // Random bytes per second
        let avg_inbound_per_sec = rand::random::<u64>() % 1000000;
        
        let metadata = proto_network::NetworkMetadata {
            total_outbound_bandwidth: Some(total_outbound),
            total_inbound_bandwidth: Some(total_inbound),
            average_outbound_bandwidth_per_second: Some(avg_outbound_per_sec),
            average_inbound_bandwidth_per_second: Some(avg_inbound_per_sec),
        };
        
        Ok(Response::new(metadata))
    }
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse().unwrap();
	
	// Initialize services
	let greeter = MyGreeter::default();
	let compute_service = ComputeServiceImpl::default();
	let memory_service = MemoryServiceImpl::default();
	let storage_service = StorageServiceImpl::default();
	let network_service = NetworkServiceImpl::default();
	
	// Initialize container services
	let docker_service = std::sync::Arc::new(DockerServiceImpl::new());
	let docker_service_clone = DockerServiceImpl::new(); // For the gRPC server
	let container_stats_service = ContainerStatsServiceImpl::new(docker_service.clone());

	let reflection_service = tonic_reflection::server::Builder::configure()
		.register_encoded_file_descriptor_set(proto_memory::FILE_DESCRIPTOR_SET)
		.build()
		.unwrap();

	println!("Infralink Worker listening on {}", addr);
	println!("Available services:");
	println!("  - Greeter (Hello World)");
	println!("  - Compute (CPU metrics)");
	println!("  - Memory (RAM and swap metrics)");
	println!("  - Storage (Disk usage metrics)");
	println!("  - Network (Bandwidth metrics)");
	println!("  - Docker (Container management)");
	println!("  - ContainerStats (Container monitoring)");

	Server::builder()
		.add_service(GreeterServer::new(greeter))
		.add_service(proto_compute::compute_service_server::ComputeServiceServer::new(compute_service))
		.add_service(proto_memory::memory_service_server::MemoryServiceServer::new(memory_service))
		.add_service(proto_storage::storage_service_server::StorageServiceServer::new(storage_service))
		.add_service(proto_network::network_service_server::NetworkServiceServer::new(network_service))
		.add_service(docker::docker_service_server::DockerServiceServer::new(docker_service_clone))
		.add_service(stats::container_stats_service_server::ContainerStatsServiceServer::new(container_stats_service))
		.add_service(reflection_service)
		.serve(addr)
		.await?;

	Ok(())
}
