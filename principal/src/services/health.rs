use reqwest::{Client, Error};
use std::time::Duration;
use tokio::time::sleep;

use crate::models::health_check::HealthCheck;
use crate::models::worker::Worker;

// This function creates a health check loop for a given HealthCheckConfig.
pub async fn create_health_check(worker: Worker, config: HealthCheck) {
	loop {
		sleep(Duration::from_millis(config.interval)).await;

		match config.r#type {
			crate::models::health_check::HealthCheckType::HTTPS => {}
			crate::models::health_check::HealthCheckType::HTTP => {
				if let Err(e) = run_http_health_check(&worker, &config).await {
					tracing::error!("Error: {:?}", e);
				}
			}
			crate::models::health_check::HealthCheckType::TCP => {}
		}
	}
}

// This function performs an HTTP health check based on the provided HealthCheckConfig.
async fn run_http_health_check(worker: &Worker, config: &HealthCheck) -> Result<(), Error> {
	let url = format!(
		"http://{}:{}/{}",
		worker.network.primary_ipv4,
		config.port,
		config.path.strip_prefix("/").unwrap()
	);

	let client = Client::builder()
		.timeout(Duration::from_millis(config.timeout))
		.build()?;

	let response = client.get(&url).send().await?;

	if response.status().is_success() {
		// Write to our database that the worker is healthy.
		// tracing::info!("{} is healthy", config.);
	} else {
		// Write to our database that the worker is healthy.
		// tracing::warn!("{} is unhealthy", );
	}

	Ok(())
}
