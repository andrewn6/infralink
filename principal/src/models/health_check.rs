use serde::{Deserialize, Serialize};

// Define a HealthCheckConfig struct for holding health check configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthCheck {
	// <origin>/{path}
	pub path: String,
	// Exposed port for the end-user's application to run the health check for (e.g. 3000).
	pub port: u64,
	// The method for the health check, if applicable.
	pub method: Option<HttpMethod>,
	// Skip TLS verification for HTTPS health checks, if applicable.
	pub tls_skip_verification: Option<bool>,
	// Grace period for the health check - it's the time to wait for the application to start before running the health check. Measured in milliseconds.
	pub grace_period: u64,
	// Interval between health checks. Measured in milliseconds. Minimum value is 10000.
	pub interval: u64,
	// Timeout for the health check. Measured in milliseconds.
	pub timeout: u64,
	// Maximum number of failed health checks before the worker is considered unhealthy.
	pub max_failures: u64,
	// Type of health check.
	pub r#type: HealthCheckType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HttpMethod {
	GET,
	POST,
	PUT,
	DELETE,
	PATCH,
	OPTIONS,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HealthCheckType {
	HTTPS,
	HTTP,
	TCP,
}
