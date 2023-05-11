use std::time::Duration;

use models::models::health_check::HealthCheck;
use models::models::worker::Worker;
use redis::cluster_async::ClusterConnection;
use redis::AsyncCommands;
use reqwest::{Client, Error};
use tokio::time::sleep;

// This function creates a health check loop for a given HealthCheckConfig.
pub async fn schedule_health_checks(
	connection: &mut ClusterConnection,
	worker: Worker,
	configs: Vec<HealthCheck>,
) -> Result<(), Error> {
	let mut handles = vec![];

	for config in configs {
		let mut connection_clone = connection.clone();
		let worker_clone = worker.clone();
		let config_clone = config.clone();

		let handle = tokio::spawn(async move {
			loop {
				sleep(Duration::from_millis(config_clone.interval)).await;
				match config_clone.r#type {
					models::models::health_check::HealthCheckType::HTTPS => {}
					models::models::health_check::HealthCheckType::HTTP => {
						if let Err(e) = run_http_health_check(
							&mut connection_clone,
							&worker_clone,
							&config_clone,
						)
						.await
						{
							tracing::error!("Error: {:?}", e);
						}
					}
					models::models::health_check::HealthCheckType::TCP => {}
				}
			}
		});
		handles.push(handle);
	}

	for handle in handles {
		let _ = handle.await;
	}

	Ok(())
}

// This function performs an HTTP health check based on the provided HealthCheckConfig.
async fn run_http_health_check(
	connection: &mut ClusterConnection,
	worker: &Worker,
	config: &HealthCheck,
) -> Result<(), Error> {
	let url = format!(
		"http://{}:{}/{}",
		worker.network.primary_ipv4,
		config.port,
		config.path.strip_prefix("/").unwrap()
	);

	let client = Client::builder()
		.timeout(Duration::from_millis(config.timeout))
		.build()?;

	let response = client
		.get(&url)
		.header("Host", "edge.dimension.dev")
		.send()
		.await?;

	if response.status().is_success() {
		// Mark the instance as available on our database.
		let _: () = connection
			.set(format!("worker:{}:available", worker.id), false)
			.await
			.unwrap();
	} else {
		// Mark the instance as unavailable on our database.
		let _: () = connection
			.set(format!("worker:{}:available", worker.id), false)
			.await
			.unwrap();
	}

	Ok(())
}
