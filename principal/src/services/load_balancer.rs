use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::services::discovery::{Endpoint, LoadBalancingPolicy};

/// Load balancer for distributing traffic across service endpoints
#[derive(Debug, Clone)]
pub struct LoadBalancer {
    /// Load balancing rules per service
    rules: HashMap<String, LoadBalancingRule>,
    /// Connection tracking for load balancing decisions
    connections: HashMap<String, ConnectionState>,
    /// Health check configuration
    health_checks: HashMap<String, HealthCheckConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingRule {
    pub service_name: String,
    pub namespace: String,
    pub policy: LoadBalancingPolicy,
    pub session_affinity: SessionAffinity,
    pub health_check_config: Option<String>, // Reference to health check config
    pub timeout_seconds: u32,
    pub retry_policy: RetryPolicy,
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionAffinity {
    None,
    ClientIP { timeout_seconds: u32 },
    Cookie { name: String, ttl_seconds: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_strategy: BackoffStrategy,
    pub retry_conditions: Vec<RetryCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed { delay_ms: u64 },
    Linear { initial_delay_ms: u64, increment_ms: u64 },
    Exponential { initial_delay_ms: u64, multiplier: f64, max_delay_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    HttpStatus(u16),
    HttpStatusRange { min: u16, max: u16 },
    Timeout,
    ConnectionError,
    ResponseError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_seconds: u32,
    pub max_requests: u32,
}

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub client_ip: String,
    pub target_endpoint: String,
    pub established_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub request_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub name: String,
    pub path: String,
    pub port: u16,
    pub protocol: HealthCheckProtocol,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
    pub headers: HashMap<String, String>,
    pub expected_codes: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckProtocol {
    HTTP,
    HTTPS,
    TCP,
    UDP,
    GRPC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingResult {
    pub selected_endpoint: Endpoint,
    pub routing_decision: RoutingDecision,
    pub session_info: Option<SessionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub algorithm_used: String,
    pub total_candidates: usize,
    pub selection_reason: String,
    pub load_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub affinity_type: String,
    pub expires_at: DateTime<Utc>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            connections: HashMap::new(),
            health_checks: HashMap::new(),
        }
    }

    /// Add or update load balancing rule for a service
    pub fn set_rule(&mut self, rule: LoadBalancingRule) {
        let key = format!("{}.{}", rule.service_name, rule.namespace);
        self.rules.insert(key, rule);
    }

    /// Select an endpoint for a request based on load balancing policy
    pub fn select_endpoint(
        &mut self,
        service_name: &str,
        namespace: &str,
        endpoints: &[Endpoint],
        client_ip: &str,
        session_id: Option<&str>,
    ) -> Option<LoadBalancingResult> {
        let key = format!("{}.{}", service_name, namespace);
        let rule = self.rules.get(&key)?.clone();
        
        if endpoints.is_empty() {
            return None;
        }

        // Filter healthy endpoints
        let healthy_endpoints: Vec<&Endpoint> = endpoints.iter()
            .filter(|ep| ep.ready && ep.serving && !ep.terminating)
            .collect();

        if healthy_endpoints.is_empty() {
            return None;
        }

        // Check session affinity first
        if let Some(endpoint) = self.check_session_affinity(&rule.session_affinity, client_ip, session_id, &healthy_endpoints) {
            return Some(LoadBalancingResult {
                selected_endpoint: endpoint.clone(),
                routing_decision: RoutingDecision {
                    algorithm_used: "session_affinity".to_string(),
                    total_candidates: healthy_endpoints.len(),
                    selection_reason: "Session affinity match".to_string(),
                    load_factor: None,
                },
                session_info: self.create_session_info(&rule.session_affinity, client_ip),
            });
        }

        // Apply load balancing policy
        let selected_endpoint = match &rule.policy {
            LoadBalancingPolicy::RoundRobin => {
                self.round_robin_select(&healthy_endpoints, service_name, namespace)
            }
            LoadBalancingPolicy::LeastConnections => {
                self.least_connections_select(&healthy_endpoints)
            }
            LoadBalancingPolicy::Random => {
                self.random_select(&healthy_endpoints)
            }
            LoadBalancingPolicy::IPHash => {
                self.ip_hash_select(&healthy_endpoints, client_ip)
            }
            LoadBalancingPolicy::WeightedRoundRobin { weights } => {
                self.weighted_round_robin_select(&healthy_endpoints, weights)
            }
        }?;

        // Track connection
        self.track_connection(client_ip, &selected_endpoint);

        Some(LoadBalancingResult {
            selected_endpoint: selected_endpoint.clone(),
            routing_decision: RoutingDecision {
                algorithm_used: format!("{:?}", rule.policy),
                total_candidates: healthy_endpoints.len(),
                selection_reason: "Load balancing policy selection".to_string(),
                load_factor: self.calculate_load_factor(&selected_endpoint),
            },
            session_info: self.create_session_info(&rule.session_affinity, client_ip),
        })
    }

    fn check_session_affinity<'a>(
        &self,
        affinity: &SessionAffinity,
        client_ip: &str,
        _session_id: Option<&str>,
        endpoints: &[&'a Endpoint],
    ) -> Option<&'a Endpoint> {
        match affinity {
            SessionAffinity::None => None,
            SessionAffinity::ClientIP { .. } => {
                // Find existing connection for this client IP
                self.connections.values()
                    .find(|conn| conn.client_ip == client_ip)
                    .and_then(|conn| {
                        endpoints.iter().find(|ep| ep.ip == conn.target_endpoint).copied()
                    })
            }
            SessionAffinity::Cookie { .. } => {
                // In a real implementation, this would check cookie-based affinity
                // For now, return None
                None
            }
        }
    }

    fn round_robin_select<'a>(&self, endpoints: &[&'a Endpoint], service_name: &str, namespace: &str) -> Option<&'a Endpoint> {
        if endpoints.is_empty() {
            return None;
        }
        
        // Simple hash-based round-robin
        let hash = self.hash_string(&format!("{}.{}", service_name, namespace));
        let index = hash % endpoints.len();
        Some(endpoints[index])
    }

    fn least_connections_select<'a>(&self, endpoints: &[&'a Endpoint]) -> Option<&'a Endpoint> {
        let mut min_connections = u64::MAX;
        let mut selected_endpoint = None;

        for endpoint in endpoints {
            let connection_count = self.connections.values()
                .filter(|conn| conn.target_endpoint == endpoint.ip)
                .count() as u64;

            if connection_count < min_connections {
                min_connections = connection_count;
                selected_endpoint = Some(*endpoint);
            }
        }

        selected_endpoint
    }

    fn random_select<'a>(&self, endpoints: &[&'a Endpoint]) -> Option<&'a Endpoint> {
        if endpoints.is_empty() {
            return None;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..endpoints.len());
        Some(endpoints[index])
    }

    fn ip_hash_select<'a>(&self, endpoints: &[&'a Endpoint], client_ip: &str) -> Option<&'a Endpoint> {
        if endpoints.is_empty() {
            return None;
        }

        let hash = self.hash_string(client_ip);
        let index = hash % endpoints.len();
        Some(endpoints[index])
    }

    fn weighted_round_robin_select<'a>(&self, endpoints: &[&'a Endpoint], weights: &HashMap<String, u32>) -> Option<&'a Endpoint> {
        if endpoints.is_empty() {
            return None;
        }

        // Calculate total weight
        let total_weight: u32 = endpoints.iter()
            .map(|ep| weights.get(&ep.ip).copied().unwrap_or(1))
            .sum();

        if total_weight == 0 {
            return self.round_robin_select(endpoints, "", "");
        }

        // Generate random number based on total weight
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut target = rng.gen_range(0..total_weight);

        // Find the endpoint based on weight distribution
        for endpoint in endpoints {
            let weight = weights.get(&endpoint.ip).copied().unwrap_or(1);
            if target < weight {
                return Some(endpoint);
            }
            target -= weight;
        }

        // Fallback to first endpoint
        Some(endpoints[0])
    }

    fn track_connection(&mut self, client_ip: &str, endpoint: &Endpoint) {
        let connection_id = format!("{}:{}", client_ip, endpoint.ip);
        let now = Utc::now();

        self.connections.insert(connection_id, ConnectionState {
            client_ip: client_ip.to_string(),
            target_endpoint: endpoint.ip.clone(),
            established_at: now,
            last_activity: now,
            request_count: 1,
            error_count: 0,
        });
    }

    fn calculate_load_factor(&self, endpoint: &Endpoint) -> Option<f64> {
        let connection_count = self.connections.values()
            .filter(|conn| conn.target_endpoint == endpoint.ip)
            .count();

        // Simple load factor based on connection count
        // In reality, this would consider CPU, memory, and other metrics
        Some(connection_count as f64 / 100.0) // Normalize to 0-1 scale
    }

    fn create_session_info(&self, affinity: &SessionAffinity, client_ip: &str) -> Option<SessionInfo> {
        match affinity {
            SessionAffinity::None => None,
            SessionAffinity::ClientIP { timeout_seconds } => {
                Some(SessionInfo {
                    session_id: self.hash_string(client_ip).to_string(),
                    affinity_type: "client_ip".to_string(),
                    expires_at: Utc::now() + chrono::Duration::seconds(*timeout_seconds as i64),
                })
            }
            SessionAffinity::Cookie { name, ttl_seconds } => {
                Some(SessionInfo {
                    session_id: uuid::Uuid::new_v4().to_string(),
                    affinity_type: format!("cookie:{}", name),
                    expires_at: Utc::now() + chrono::Duration::seconds(*ttl_seconds as i64),
                })
            }
        }
    }

    fn hash_string(&self, input: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Add health check configuration
    pub fn add_health_check(&mut self, config: HealthCheckConfig) {
        self.health_checks.insert(config.name.clone(), config);
    }

    /// Remove stale connections
    pub fn cleanup_connections(&mut self, max_idle_seconds: u64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_idle_seconds as i64);
        
        self.connections.retain(|_, conn| conn.last_activity > cutoff);
    }

    /// Get load balancing statistics
    pub fn get_stats(&self) -> LoadBalancerStats {
        let total_connections = self.connections.len();
        let active_connections = self.connections.values()
            .filter(|conn| {
                let five_minutes_ago = Utc::now() - chrono::Duration::minutes(5);
                conn.last_activity > five_minutes_ago
            })
            .count();

        LoadBalancerStats {
            total_rules: self.rules.len(),
            total_connections,
            active_connections,
            health_checks: self.health_checks.len(),
        }
    }

    /// Update connection activity
    pub fn update_connection_activity(&mut self, client_ip: &str, endpoint_ip: &str, success: bool) {
        let connection_id = format!("{}:{}", client_ip, endpoint_ip);
        
        if let Some(conn) = self.connections.get_mut(&connection_id) {
            conn.last_activity = Utc::now();
            conn.request_count += 1;
            if !success {
                conn.error_count += 1;
            }
        }
    }

    /// Check if circuit breaker should trip for an endpoint
    pub fn check_circuit_breaker(&self, _endpoint: &Endpoint, _rule: &LoadBalancingRule) -> bool {
        // TODO: Implement circuit breaker logic
        // For now, always allow traffic
        true
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub total_rules: usize,
    pub total_connections: usize,
    pub active_connections: usize,
    pub health_checks: usize,
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}