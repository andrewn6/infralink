use dotenv_codegen::dotenv;
use models::models::health_check::{HealthCheck, HealthCheckType, HttpMethod};
use models::models::network::Network;
use redis::cluster_async::ClusterConnection;
use redis::AsyncCommands;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Struct to hold information about an ongoing health check.
pub struct HealthCheckTask {
	// The JoinHandle for the health check loop.
	pub handle: tokio::task::JoinHandle<()>,
	// Unique ID of the health check.
	pub health_check_id: String,
}

/// Struct to hold information about a worker that we need to perform health checks on.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerInfo {
	// Worker ID
	pub id: u64,
	// Worker networking information.
	pub network: Network,
}

// This function creates a health check loop for a given HealthCheckConfig.
pub async fn schedule_health_checks(
	connection: &mut ClusterConnection,
	project_id: u64,
	worker: WorkerInfo,
	configs: Vec<HealthCheck>,
	tasks_map: Arc<Mutex<HashMap<String, HealthCheckTask>>>, // added this
) -> Vec<String> {
	let mut tasks = vec![];

	for config in configs {
		let mut connection_clone = connection.clone();
		let worker_clone = worker.clone();
		let config_clone = config.clone();
		let region = dotenv!("REGION");

		let uuid = Uuid::new_v4();
		let key = format!(
			"pj:{}:wkr:{}:{}:{}",
			project_id, worker_clone.id, region, uuid
		);

		let key_clone = key.clone(); // clone the key here

		let handle = tokio::spawn(async move {
			// Wait for the grace period before starting the health checks.
			sleep(Duration::from_millis(config_clone.grace_period)).await;

			let mut failure_count = 0;

			loop {
				// Run the health check and increment the failure count if it fails.
				if let Err(e) = run_http_health_check(
					&mut connection_clone,
					project_id,
					&worker_clone,
					&config_clone,
					&uuid.to_string(),
					region,
				)
				.await
				{
					tracing::error!("Error: {:?}", e);
					failure_count += 1;
				} else {
					failure_count = 0;
				}

				// Mark the worker as unavailable if the failure count exceeds the maximum.
				if failure_count > config_clone.max_failures {
					let _: () = connection_clone
						.hset(&key_clone, "available", false)
						.await
						.unwrap();
				}

				// Wait for the interval before running the next health check.
				sleep(Duration::from_millis(config_clone.interval)).await;
			}
		});

		tasks_map.lock().unwrap().insert(
			key.clone(),
			HealthCheckTask {
				handle,
				health_check_id: key.clone(),
			},
		);

		tasks.push(key);
	}

	tasks
}

/// This function performs an HTTP health check based on the provided HealthCheckConfig.
/// It sends an HTTP request to the specified worker and updates its status in the database based on the response.
async fn run_http_health_check(
	connection: &mut ClusterConnection,
	project_id: u64,
	worker: &WorkerInfo,
	config: &HealthCheck,
	uuid: &str,
	region: &str,
) -> Result<(), Error> {
	// Construct the URL based on the provided configuration.
	let url = format!(
		"http{}://{}:{}/{}",
		worker.network.primary_ipv4,
		// to check if the health check requires https, we can match on the type
		if config.r#type == HealthCheckType::HTTPS {
			"s"
		} else {
			""
		},
		config.port,
		config.path.strip_prefix("/").unwrap_or(&config.path)
	);

	// Create an HTTP client with the specified timeout.
	let client = Client::builder()
		.timeout(Duration::from_millis(config.timeout))
		.danger_accept_invalid_certs(config.tls_skip_verification.unwrap_or(false))
		.build()?;

	// Construct the headers based on the provided configuration.
	let mut headers = HeaderMap::new();
	if let Some(header_vec) = &config.headers {
		for header in header_vec {
			headers.insert(
				HeaderName::from_bytes(header.key.as_bytes()).unwrap(),
				HeaderValue::from_str(&header.value).unwrap(),
			);
		}
	}

	// Send the HTTP request and await the response.
	let response = match config.method {
		Some(HttpMethod::GET) => client.get(&url).headers(headers).send().await?,
		Some(HttpMethod::POST) => client.post(&url).headers(headers).send().await?,
		// Add more HTTP methods as needed.
		_ => client.get(&url).headers(headers).send().await?,
	};

	// Update the status of the worker in the database based on the response.
	let key = format!("pj:{}:wkr:{}:{}:{}", project_id, worker.id, region, uuid);

	let _: () = connection
		.hset(&key, "available", response.status().is_success())
		.await
		.unwrap();

	// Record the time of the last health check.
	let _: () = connection
		.hset(&key, "last_health_check", chrono::Utc::now().to_rfc3339())
		.await
		.unwrap();

	Ok(())
}

pub fn stop_health_check(
	tasks_map: Arc<Mutex<HashMap<String, HealthCheckTask>>>,
	health_check_id: String,
) {
	let mut map = tasks_map.lock().unwrap();
	if let Some(task) = map.remove(&health_check_id) {
		task.handle.abort(); // This will stop the health check.
	} else {
		println!("No health check found with ID {}", health_check_id);
	}
}
