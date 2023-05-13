use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use dotenv::dotenv;
use models::models::health_check::{Header, HealthCheck, HealthCheckType, HttpMethod};
use models::models::network::Network;

pub mod db;
pub mod health_check;

use health_check::{schedule_health_checks, HealthCheckTask, WorkerInfo};

#[tokio::main]
pub async fn main() {
	dotenv().unwrap();

	let mut connection = db::connection().await.unwrap();

	let tasks_map: Arc<Mutex<HashMap<String, HealthCheckTask>>> =
		Arc::new(Mutex::new(HashMap::new()));

	schedule_health_checks(
		&mut connection,
		1,
		WorkerInfo {
			id: 123,
			network: Network {
				primary_ipv4: String::from("139.180.143.66"),
				primary_ipv6: String::new(),
			},
		},
		vec![HealthCheck {
			path: "/health".to_string(),
			port: 80,
			method: Some(HttpMethod::GET),
			tls_skip_verification: None,
			grace_period: 0,
			interval: 4000,
			timeout: 10000,
			max_failures: 2,
			r#type: HealthCheckType::HTTP,
			headers: Some(vec![Header {
				key: "Host".to_string(),
				value: "edge.dimension.dev".to_string(),
			}]),
		}],
		tasks_map.clone(),
	)
	.await;
}
