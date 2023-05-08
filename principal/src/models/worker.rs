use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

use super::cloud_provider::CloudProvider;
use super::health_check::HealthCheck;
use super::instance::Instance;
use super::instance_state::InstanceState;
use super::metrics::Metrics;
use super::network::Network;
use super::region::Region;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Worker {
	pub id: String,
	pub network: Network,
	pub provider: CloudProvider,
	pub region: Region,
	pub instance: Instance,
	pub metrics: Metrics,
	pub state: InstanceState,
	pub volumes: Vec<String>,
	pub last_updated: DateTime<Utc>,
	pub last_health_check: HealthCheck,
}
