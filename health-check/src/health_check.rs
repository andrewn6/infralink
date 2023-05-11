use std::time::Duration;

use models::models::health_check::HealthCheck;
use models::models::worker::Worker;
use reqwest::{Client, Error};
use tokio::time::sleep;

// This function creates a health check loop for a given HealthCheckConfig.
pub async fn create_health_check(
	connection: &mut redis::Connection,
	worker: Worker,
	config: HealthCheck,
) {
	loop {
		sleep(Duration::from_millis(config.interval)).await;

		match config.r#type {
			models::models::health_check::HealthCheckType::HTTPS => {}
			models::models::health_check::HealthCheckType::HTTP => {
				if let Err(e) = run_http_health_check(connection, &worker, &config).await {
					tracing::error!("Error: {:?}", e);
				}
			}
			models::models::health_check::HealthCheckType::TCP => {}
		}
	}
}

// This function performs an HTTP health check based on the provided HealthCheckConfig.
async fn run_http_health_check(
	connection: &mut redis::Connection,
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
		// Write to our database that the worker is healthy.
		redis::cmd("SET")
			.arg(format!("worker:{}:up", worker.id))
			.arg("true")
			.query::<()>(connection)
			.unwrap();

		tracing::info!("{} is healthy", worker.id);
	} else {
		redis::cmd("SET")
			.arg(format!("worker:{}:up", worker.id))
			.arg("false")
			.query::<()>(connection)
			.unwrap();

		// Write to our database that the worker is healthy.
		tracing::warn!("{} is unhealthy", worker.id);
	}

	Ok(())
}
