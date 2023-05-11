use chrono::Utc;
use dotenv::dotenv;
use models::models::cloud_provider::CloudProvider;
use models::models::health_check::{Header, HealthCheck, HealthCheckType, HttpMethod};
use models::models::instance::Instance;
use models::models::instance_state::InstanceState;
use models::models::metrics::Metrics;
use models::models::network::Network;
use models::models::region::Region;
use models::models::volume::{Volume, VolumeTier, VolumeType};
use models::models::worker::Worker;

pub mod db;
pub mod health_check;

use health_check::schedule_health_checks;

#[tokio::main]
pub async fn main() {
	dotenv().unwrap();

	let mut connection = db::connection().await.unwrap();

	schedule_health_checks(
		&mut connection,
		Worker {
			id: 123.to_string(),
			network: Network {
				primary_ipv4: String::from("139.180.143.66"),
				primary_ipv6: String::new(),
			},
			provider: CloudProvider::Vultr,
			region: Region::NewYork,
			instance: Instance {
				vcpu: 1,
				memory: 1,
				boot_volume: Volume {
					id: 1,
					used: 10,
					total: 100,
					r#type: VolumeType::NVME,
					tier: VolumeTier::HighPerformance,
				},
			},
			metrics: Metrics {
				cpu: 4.0,
				memory: 512.0,
				disk: 100.0,
				network: 100.0,
				time: Utc::now(),
				workload: 10.0,
			},
			state: InstanceState::Running,
			volumes: vec![],
			last_updated: Utc::now(),
			last_health_check: None,
		},
		vec![HealthCheck {
			path: "/health".to_string(),
			port: 80,
			method: Some(HttpMethod::GET),
			tls_skip_verification: Some(false),
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
	)
	.await
	.unwrap();
}
