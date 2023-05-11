use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metrics {
	pub cpu: f64,
	pub memory: f64,
	pub disk: f64,
	pub network: f64,
	pub workload: f64,
	pub time: DateTime<Utc>,
}
