use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::scale::scale::{Service, Pod, ServicePort, IntOrString};

/// Service discovery and DNS resolution for Infralink
#[derive(Debug, Clone)]
pub struct ServiceDiscovery {
    /// Maps service names to their endpoints
    service_endpoints: HashMap<String, ServiceEndpoints>,
    /// DNS records for service discovery
    dns_records: HashMap<String, DnsRecord>,
    /// Service registry for external services
    external_services: HashMap<String, ExternalService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    pub service_id: String,
    pub service_name: String,
    pub namespace: String,
    pub endpoints: Vec<Endpoint>,
    pub ports: Vec<ServicePort>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub pod_id: String,
    pub ip: String,
    pub port: u16,
    pub ready: bool,
    pub serving: bool,
    pub terminating: bool,
    pub node_name: Option<String>,
    pub zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub namespace: String,
    pub record_type: DnsRecordType,
    pub ttl: u32,
    pub values: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DnsRecordType {
    A,
    AAAA,
    CNAME,
    SRV,
    TXT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalService {
    pub name: String,
    pub namespace: String,
    pub external_name: String,
    pub ports: Vec<ServicePort>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLookupResult {
    pub service_name: String,
    pub namespace: String,
    pub endpoints: Vec<Endpoint>,
    pub load_balancing_policy: LoadBalancingPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingPolicy {
    RoundRobin,
    LeastConnections,
    Random,
    IPHash,
    WeightedRoundRobin { weights: HashMap<String, u32> },
}

impl ServiceDiscovery {
    pub fn new() -> Self {
        Self {
            service_endpoints: HashMap::new(),
            dns_records: HashMap::new(),
            external_services: HashMap::new(),
        }
    }

    /// Register a service and its endpoints
    pub fn register_service(&mut self, service: &Service, pods: Vec<&Pod>) -> Result<(), Box<dyn std::error::Error>> {
        let endpoints = self.create_endpoints_from_pods(service, pods)?;
        
        let service_endpoints = ServiceEndpoints {
            service_id: service.id.clone(),
            service_name: service.name.clone(),
            namespace: service.namespace.clone(),
            endpoints,
            ports: service.spec.ports.clone(),
            last_updated: Utc::now(),
        };

        // Create DNS records for the service
        self.create_service_dns_records(service)?;
        
        let endpoints_count = service_endpoints.endpoints.len();
        let key = self.service_key(&service.name, &service.namespace);
        self.service_endpoints.insert(key, service_endpoints);
        
        println!("Registered service {} in namespace {} with {} endpoints", 
                 service.name, service.namespace, endpoints_count);
        
        Ok(())
    }

    /// Update service endpoints when pods change
    pub fn update_service_endpoints(&mut self, service: &Service, pods: Vec<&Pod>) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.service_key(&service.name, &service.namespace);
        let new_endpoints = self.create_endpoints_from_pods(service, pods)?;
        let endpoints_count = new_endpoints.len();
        
        if let Some(service_endpoints) = self.service_endpoints.get_mut(&key) {
            service_endpoints.endpoints = new_endpoints;
            service_endpoints.last_updated = Utc::now();
            
            println!("Updated endpoints for service {} - now has {} endpoints", 
                     service.name, endpoints_count);
        }
        
        Ok(())
    }

    /// Get service endpoints by name (searches all namespaces)
    pub fn get_service_endpoints(&self, service_name: &str) -> Vec<Endpoint> {
        self.service_endpoints
            .values()
            .filter(|endpoints| endpoints.service_name == service_name)
            .flat_map(|endpoints| &endpoints.endpoints)
            .filter(|ep| ep.ready && ep.serving && !ep.terminating)
            .cloned()
            .collect()
    }

    /// Register a simple service endpoint (for examples and testing)
    pub fn register_simple_service(
        &mut self,
        service_name: String,
        ip: String,
        port: u16,
        metadata: HashMap<String, String>,
    ) {
        let endpoint = Endpoint {
            pod_id: format!("pod-{}", uuid::Uuid::new_v4()),
            ip,
            port,
            ready: true,
            serving: true,
            terminating: false,
            node_name: Some("local-node".to_string()),
            zone: Some("local".to_string()),
        };

        let service_endpoints = ServiceEndpoints {
            service_id: uuid::Uuid::new_v4().to_string(),
            service_name: service_name.clone(),
            namespace: "default".to_string(),
            endpoints: vec![endpoint],
            ports: vec![ServicePort {
                name: Some("http".to_string()),
                port,
                target_port: IntOrString::Int(port as u32),
                protocol: "TCP".to_string(),
                node_port: None,
            }],
            last_updated: Utc::now(),
        };

        let key = self.service_key(&service_name, "default");
        self.service_endpoints.insert(key, service_endpoints);
    }

    /// Look up service by name and namespace
    pub fn lookup_service(&self, service_name: &str, namespace: &str) -> Option<ServiceLookupResult> {
        let key = self.service_key(service_name, namespace);
        
        self.service_endpoints.get(&key).map(|endpoints| {
            ServiceLookupResult {
                service_name: service_name.to_string(),
                namespace: namespace.to_string(),
                endpoints: endpoints.endpoints.iter()
                    .filter(|ep| ep.ready && ep.serving && !ep.terminating)
                    .cloned()
                    .collect(),
                load_balancing_policy: LoadBalancingPolicy::RoundRobin, // Default policy
            }
        })
    }

    /// Resolve service to a specific endpoint using load balancing
    pub fn resolve_service(&self, service_name: &str, namespace: &str) -> Option<Endpoint> {
        if let Some(lookup_result) = self.lookup_service(service_name, namespace) {
            if lookup_result.endpoints.is_empty() {
                return None;
            }
            
            match lookup_result.load_balancing_policy {
                LoadBalancingPolicy::RoundRobin => {
                    // Simple round-robin implementation
                    let index = self.get_round_robin_index(service_name, namespace) % lookup_result.endpoints.len();
                    Some(lookup_result.endpoints[index].clone())
                }
                LoadBalancingPolicy::Random => {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0..lookup_result.endpoints.len());
                    Some(lookup_result.endpoints[index].clone())
                }
                LoadBalancingPolicy::LeastConnections => {
                    // For now, use round-robin as we don't track connections
                    let index = self.get_round_robin_index(service_name, namespace) % lookup_result.endpoints.len();
                    Some(lookup_result.endpoints[index].clone())
                }
                LoadBalancingPolicy::IPHash => {
                    // Simple hash-based selection
                    let hash = self.hash_service_name(service_name, namespace);
                    let index = hash % lookup_result.endpoints.len();
                    Some(lookup_result.endpoints[index].clone())
                }
                LoadBalancingPolicy::WeightedRoundRobin { weights: _ } => {
                    // For now, fallback to round-robin
                    let index = self.get_round_robin_index(service_name, namespace) % lookup_result.endpoints.len();
                    Some(lookup_result.endpoints[index].clone())
                }
            }
        } else {
            None
        }
    }

    /// Register an external service
    pub fn register_external_service(&mut self, external_service: ExternalService) -> Result<(), Box<dyn std::error::Error>> {
        // Create CNAME DNS record for external service
        let dns_record = DnsRecord {
            name: external_service.name.clone(),
            namespace: external_service.namespace.clone(),
            record_type: DnsRecordType::CNAME,
            ttl: 300, // 5 minutes
            values: vec![external_service.external_name.clone()],
            created_at: Utc::now(),
        };
        
        let dns_key = self.dns_key(&external_service.name, &external_service.namespace);
        self.dns_records.insert(dns_key, dns_record);
        
        let key = self.service_key(&external_service.name, &external_service.namespace);
        self.external_services.insert(key, external_service);
        
        Ok(())
    }

    /// Get all services in a namespace
    pub fn list_services(&self, namespace: &str) -> Vec<&ServiceEndpoints> {
        self.service_endpoints.values()
            .filter(|endpoints| endpoints.namespace == namespace)
            .collect()
    }

    /// Remove a service from discovery
    pub fn unregister_service(&mut self, service_name: &str, namespace: &str) {
        let key = self.service_key(service_name, namespace);
        self.service_endpoints.remove(&key);
        
        let dns_key = self.dns_key(service_name, namespace);
        self.dns_records.remove(&dns_key);
        
        println!("Unregistered service {} from namespace {}", service_name, namespace);
    }

    /// Create endpoints from pods that match the service selector
    fn create_endpoints_from_pods(&self, service: &Service, pods: Vec<&Pod>) -> Result<Vec<Endpoint>, Box<dyn std::error::Error>> {
        let mut endpoints = Vec::new();
        
        for pod in pods {
            // Check if pod matches service selector
            let matches_selector = service.spec.selector.iter().all(|(key, value)| {
                pod.labels.get(key) == Some(value)
            });
            
            if matches_selector {
                for port in &service.spec.ports {
                    if let Some(pod_ip) = &pod.status.pod_ip {
                        let target_port = match &port.target_port {
                            IntOrString::Int(port_num) => *port_num as u16,
                            IntOrString::String(port_name) => {
                                // Look up named port in pod spec
                                self.resolve_named_port(&pod.spec.containers, port_name)?
                            }
                        };
                        
                        endpoints.push(Endpoint {
                            pod_id: pod.id.clone(),
                            ip: pod_ip.clone(),
                            port: target_port,
                            ready: matches!(pod.status.phase, crate::scale::scale::PodPhase::Running),
                            serving: true,
                            terminating: false,
                            node_name: pod.node_id.clone(),
                            zone: None, // TODO: Extract zone from node
                        });
                    }
                }
            }
        }
        
        Ok(endpoints)
    }

    fn resolve_named_port(&self, containers: &[crate::scale::scale::ContainerSpec], port_name: &str) -> Result<u16, Box<dyn std::error::Error>> {
        for container in containers {
            for port in &container.ports {
                if port.name.as_deref() == Some(port_name) {
                    return Ok(port.container_port);
                }
            }
        }
        Err(format!("Named port '{}' not found in container specs", port_name).into())
    }

    fn create_service_dns_records(&mut self, service: &Service) -> Result<(), Box<dyn std::error::Error>> {
        // Create A record for ClusterIP services
        if let Some(cluster_ip) = &service.spec.cluster_ip {
            let dns_record = DnsRecord {
                name: service.name.clone(),
                namespace: service.namespace.clone(),
                record_type: DnsRecordType::A,
                ttl: 30, // 30 seconds for internal services
                values: vec![cluster_ip.clone()],
                created_at: Utc::now(),
            };
            
            let key = self.dns_key(&service.name, &service.namespace);
            self.dns_records.insert(key, dns_record);
        }
        
        // Create SRV records for service ports
        for port in &service.spec.ports {
            if let Some(port_name) = &port.name {
                let srv_name = format!("_{}._{}.{}", port_name, port.protocol.to_lowercase(), service.name);
                let srv_value = format!("0 5 {} {}.{}.svc.cluster.local.", 
                                       port.port, service.name, service.namespace);
                
                let srv_record = DnsRecord {
                    name: srv_name,
                    namespace: service.namespace.clone(),
                    record_type: DnsRecordType::SRV,
                    ttl: 30,
                    values: vec![srv_value],
                    created_at: Utc::now(),
                };
                
                let key = self.dns_key(&srv_record.name, &service.namespace);
                self.dns_records.insert(key, srv_record);
            }
        }
        
        Ok(())
    }

    fn service_key(&self, name: &str, namespace: &str) -> String {
        format!("{}.{}", name, namespace)
    }

    fn dns_key(&self, name: &str, namespace: &str) -> String {
        format!("{}.{}.svc.cluster.local", name, namespace)
    }

    fn get_round_robin_index(&self, service_name: &str, namespace: &str) -> usize {
        // Simple hash-based round-robin counter
        // In a real implementation, this would be a proper counter
        let hash = self.hash_service_name(service_name, namespace);
        (hash % 1000) as usize
    }

    fn hash_service_name(&self, service_name: &str, namespace: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        service_name.hash(&mut hasher);
        namespace.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Health check endpoints
    pub fn health_check_endpoints(&mut self) {
        // TODO: Implement health checking logic
        // This would periodically check endpoint health and update ready status
        println!("Health checking service endpoints...");
        
        for service_endpoints in self.service_endpoints.values_mut() {
            for endpoint in &mut service_endpoints.endpoints {
                // Mock health check - in reality, this would make HTTP/TCP requests
                endpoint.ready = true;
                endpoint.serving = true;
            }
        }
    }

    /// Get DNS records for a name
    pub fn resolve_dns(&self, name: &str, namespace: &str) -> Option<&DnsRecord> {
        let key = self.dns_key(name, namespace);
        self.dns_records.get(&key)
    }

    /// Get service statistics
    pub fn get_service_stats(&self) -> ServiceDiscoveryStats {
        let total_services = self.service_endpoints.len();
        let total_endpoints = self.service_endpoints.values()
            .map(|s| s.endpoints.len())
            .sum();
        let healthy_endpoints = self.service_endpoints.values()
            .flat_map(|s| &s.endpoints)
            .filter(|e| e.ready && e.serving)
            .count();
        
        ServiceDiscoveryStats {
            total_services,
            total_endpoints,
            healthy_endpoints,
            total_dns_records: self.dns_records.len(),
            external_services: self.external_services.len(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceDiscoveryStats {
    pub total_services: usize,
    pub total_endpoints: usize,
    pub healthy_endpoints: usize,
    pub total_dns_records: usize,
    pub external_services: usize,
}

impl Default for ServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}