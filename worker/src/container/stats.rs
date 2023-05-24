use bollard::Docker;
use bollard::errors::Error;
use bollard::container::Stats;

use tonic::{Request, Response, Status};
use futures_util::stream::TryStreamExt;

mod stats {
    include!("../stats.rs");
}


use stats::{ContainerStatsRequest, ContainerStatsResponse};

#[tonic::async_trait]
pub trait ContainerStatsService {
    async fn get_container_stats(
        &self,
        request: Request<ContainerStatsRequest>,
    ) -> Result<Response<ContainerStatsResponse>, Status>;
}

pub struct MyContainerStatsService {}

#[tonic::async_trait]
impl ContainerStatsService for MyContainerStatsService {
    async fn get_container_stats(
        &self,
        request: Request<ContainerStatsRequest>,
    ) -> Result<Response<ContainerStatsResponse>, Status> {
        let request = request.into_inner();

        let docker = Docker::connect_with_local_defaults().unwrap();

        match docker.stats(&request.container_id, None).try_collect::<Vec<Stats>>().await {
            Ok(stats) => {
                if let Some(first_stat) = stats.get(0) {
                    match process_stats(first_stat) {
                        Ok(response) => Ok(Response::new(response)),
                        Err(err) => {
                            eprintln!("Error processing stats: {:?}", err);
                            Err(Status::internal("Failed to process stats"))
                        }
                    }
                } else {
                    Err(Status::internal("No stats available"))
                }
            }
            Err(err) => {
                eprintln!("Error getting stats: {:?}", err);
                Err(Status::internal("Failed to get stats"))
            }
        }
    }
}

fn process_stats(stats: &Stats) -> Result<ContainerStatsResponse, &'static str> {
    let cpu_usage = stats.cpu_stats.cpu_usage.total_usage as f64;
    let memory_usage = stats.memory_stats.usage.unwrap_or(0) as f64;

    let network_io = match &stats.network {
        Some(network) => network.rx_bytes as f64 + network.tx_bytes as f64,
        None => 0.0,
    };

    let block_io = stats.blkio_stats.io_service_bytes_recursive.iter().flat_map(|io| io.iter()).map(|io| io.value).sum::<u64>() as f64;

    let response = ContainerStatsResponse {
        cpu_usage,
        memory_usage,
        network_io,
        block_io,
    };

    Ok(response)
}