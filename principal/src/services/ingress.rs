use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Ingress controller for managing external traffic routing
#[derive(Debug, Clone)]
pub struct IngressController {
    pub rules: Arc<Mutex<HashMap<String, IngressRule>>>,
    pub certificates: Arc<Mutex<HashMap<String, TlsCertificate>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub spec: IngressSpec,
    pub status: IngressStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressSpec {
    pub rules: Vec<IngressRuleSpec>,
    pub tls: Vec<IngressTls>,
    pub ingress_class_name: Option<String>,
    pub default_backend: Option<IngressBackend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRuleSpec {
    pub host: Option<String>,
    pub http: Option<HttpIngressRuleValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpIngressRuleValue {
    pub paths: Vec<HttpIngressPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpIngressPath {
    pub path: String,
    pub path_type: PathType,
    pub backend: IngressBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathType {
    Exact,
    Prefix,
    ImplementationSpecific,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressBackend {
    pub service: IngressServiceBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressServiceBackend {
    pub name: String,
    pub port: ServiceBackendPort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBackendPort {
    pub number: Option<u16>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressTls {
    pub hosts: Vec<String>,
    pub secret_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressStatus {
    pub load_balancer: Option<LoadBalancerStatus>,
    pub conditions: Vec<IngressCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStatus {
    pub ingress: Vec<LoadBalancerIngress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerIngress {
    pub ip: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressCondition {
    pub condition_type: String,
    pub status: String,
    pub last_transition_time: chrono::DateTime<chrono::Utc>,
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsCertificate {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub certificate: String, // PEM encoded certificate
    pub private_key: String, // PEM encoded private key
    pub hosts: Vec<String>,
    pub issuer: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMatch {
    pub rule_id: String,
    pub backend: IngressBackend,
    pub path: String,
    pub host: Option<String>,
    pub tls_enabled: bool,
}

impl IngressController {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(HashMap::new())),
            certificates: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new ingress rule
    pub fn create_ingress_rule(
        &self,
        name: String,
        namespace: String,
        spec: IngressSpec,
    ) -> Result<IngressRule, IngressError> {
        let rule_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let rule = IngressRule {
            id: rule_id.clone(),
            name: name.clone(),
            namespace: namespace.clone(),
            spec,
            status: IngressStatus {
                load_balancer: Some(LoadBalancerStatus {
                    ingress: vec![LoadBalancerIngress {
                        ip: Some("127.0.0.1".to_string()), // Mock IP for local development
                        hostname: Some(format!("{}.local", name)),
                    }],
                }),
                conditions: vec![IngressCondition {
                    condition_type: "Ready".to_string(),
                    status: "True".to_string(),
                    last_transition_time: now,
                    reason: "IngressCreated".to_string(),
                    message: "Ingress rule successfully created".to_string(),
                }],
            },
            created_at: now,
            updated_at: now,
        };

        let mut rules = self.rules.lock().unwrap();
        rules.insert(rule_id, rule.clone());

        println!("Created ingress rule: {} in namespace: {}", name, namespace);
        Ok(rule)
    }

    /// Update an existing ingress rule
    pub fn update_ingress_rule(
        &self,
        rule_id: &str,
        spec: IngressSpec,
    ) -> Result<IngressRule, IngressError> {
        let mut rules = self.rules.lock().unwrap();
        
        if let Some(rule) = rules.get_mut(rule_id) {
            rule.spec = spec;
            rule.updated_at = chrono::Utc::now();
            
            println!("Updated ingress rule: {}", rule.name);
            Ok(rule.clone())
        } else {
            Err(IngressError::RuleNotFound(rule_id.to_string()))
        }
    }

    /// Delete an ingress rule
    pub fn delete_ingress_rule(&self, rule_id: &str) -> Result<(), IngressError> {
        let mut rules = self.rules.lock().unwrap();
        
        if let Some(rule) = rules.remove(rule_id) {
            println!("Deleted ingress rule: {}", rule.name);
            Ok(())
        } else {
            Err(IngressError::RuleNotFound(rule_id.to_string()))
        }
    }

    /// Get ingress rule by ID
    pub fn get_ingress_rule(&self, rule_id: &str) -> Option<IngressRule> {
        let rules = self.rules.lock().unwrap();
        rules.get(rule_id).cloned()
    }

    /// List all ingress rules
    pub fn list_ingress_rules(&self) -> Vec<IngressRule> {
        let rules = self.rules.lock().unwrap();
        rules.values().cloned().collect()
    }

    /// List ingress rules by namespace
    pub fn list_ingress_rules_by_namespace(&self, namespace: &str) -> Vec<IngressRule> {
        let rules = self.rules.lock().unwrap();
        rules
            .values()
            .filter(|rule| rule.namespace == namespace)
            .cloned()
            .collect()
    }

    /// Route a request based on host and path
    pub fn route_request(&self, host: &str, path: &str) -> Option<RouteMatch> {
        let rules = self.rules.lock().unwrap();

        // Find the best matching rule
        for rule in rules.values() {
            for rule_spec in &rule.spec.rules {
                // Check host match
                let host_matches = rule_spec.host.as_ref()
                    .map_or(true, |rule_host| rule_host == host || rule_host == "*");

                if !host_matches {
                    continue;
                }

                // Check path match
                if let Some(http) = &rule_spec.http {
                    for http_path in &http.paths {
                        let path_matches = match http_path.path_type {
                            PathType::Exact => path == http_path.path,
                            PathType::Prefix => path.starts_with(&http_path.path),
                            PathType::ImplementationSpecific => {
                                // Use prefix matching as default
                                path.starts_with(&http_path.path)
                            }
                        };

                        if path_matches {
                            // Check if TLS is enabled for this host
                            let tls_enabled = rule.spec.tls.iter()
                                .any(|tls| tls.hosts.contains(&host.to_string()));

                            return Some(RouteMatch {
                                rule_id: rule.id.clone(),
                                backend: http_path.backend.clone(),
                                path: http_path.path.clone(),
                                host: rule_spec.host.clone(),
                                tls_enabled,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Add TLS certificate
    pub fn add_tls_certificate(
        &self,
        name: String,
        namespace: String,
        certificate: String,
        private_key: String,
        hosts: Vec<String>,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<TlsCertificate, IngressError> {
        let cert_id = Uuid::new_v4().to_string();
        
        let tls_cert = TlsCertificate {
            id: cert_id.clone(),
            name: name.clone(),
            namespace,
            certificate,
            private_key,
            hosts,
            issuer: "infralink-ca".to_string(),
            created_at: chrono::Utc::now(),
            expires_at,
        };

        let mut certificates = self.certificates.lock().unwrap();
        certificates.insert(cert_id, tls_cert.clone());

        println!("Added TLS certificate: {}", name);
        Ok(tls_cert)
    }

    /// Get TLS certificate by name
    pub fn get_tls_certificate(&self, name: &str, namespace: &str) -> Option<TlsCertificate> {
        let certificates = self.certificates.lock().unwrap();
        certificates
            .values()
            .find(|cert| cert.name == name && cert.namespace == namespace)
            .cloned()
    }

    /// List expired certificates
    pub fn list_expired_certificates(&self) -> Vec<TlsCertificate> {
        let certificates = self.certificates.lock().unwrap();
        let now = chrono::Utc::now();
        
        certificates
            .values()
            .filter(|cert| cert.expires_at < now)
            .cloned()
            .collect()
    }

    /// Get ingress health summary
    pub fn get_health_summary(&self) -> IngressHealthSummary {
        let rules = self.rules.lock().unwrap();
        let certificates = self.certificates.lock().unwrap();
        let now = chrono::Utc::now();

        let expired_certs = certificates
            .values()
            .filter(|cert| cert.expires_at < now)
            .count();

        let expiring_soon_certs = certificates
            .values()
            .filter(|cert| {
                let days_until_expiry = (cert.expires_at - now).num_days();
                days_until_expiry > 0 && days_until_expiry <= 30
            })
            .count();

        IngressHealthSummary {
            total_rules: rules.len(),
            active_rules: rules.values().filter(|rule| {
                rule.status.conditions.iter()
                    .any(|condition| condition.condition_type == "Ready" && condition.status == "True")
            }).count(),
            total_certificates: certificates.len(),
            expired_certificates: expired_certs,
            expiring_soon_certificates: expiring_soon_certs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IngressHealthSummary {
    pub total_rules: usize,
    pub active_rules: usize,
    pub total_certificates: usize,
    pub expired_certificates: usize,
    pub expiring_soon_certificates: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum IngressError {
    #[error("Ingress rule not found: {0}")]
    RuleNotFound(String),
    #[error("Certificate not found: {0}")]
    CertificateNotFound(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Backend service not available: {0}")]
    BackendNotAvailable(String),
}

impl Default for IngressController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ingress_rule() {
        let controller = IngressController::new();
        
        let spec = IngressSpec {
            rules: vec![IngressRuleSpec {
                host: Some("example.com".to_string()),
                http: Some(HttpIngressRuleValue {
                    paths: vec![HttpIngressPath {
                        path: "/api".to_string(),
                        path_type: PathType::Prefix,
                        backend: IngressBackend {
                            service: IngressServiceBackend {
                                name: "api-service".to_string(),
                                port: ServiceBackendPort {
                                    number: Some(8080),
                                    name: None,
                                },
                            },
                        },
                    }],
                }),
            }],
            tls: vec![],
            ingress_class_name: None,
            default_backend: None,
        };

        let rule = controller.create_ingress_rule(
            "test-ingress".to_string(),
            "default".to_string(),
            spec,
        ).unwrap();

        assert_eq!(rule.name, "test-ingress");
        assert_eq!(rule.namespace, "default");
    }

    #[test]
    fn test_route_request() {
        let controller = IngressController::new();
        
        let spec = IngressSpec {
            rules: vec![IngressRuleSpec {
                host: Some("example.com".to_string()),
                http: Some(HttpIngressRuleValue {
                    paths: vec![HttpIngressPath {
                        path: "/api".to_string(),
                        path_type: PathType::Prefix,
                        backend: IngressBackend {
                            service: IngressServiceBackend {
                                name: "api-service".to_string(),
                                port: ServiceBackendPort {
                                    number: Some(8080),
                                    name: None,
                                },
                            },
                        },
                    }],
                }),
            }],
            tls: vec![],
            ingress_class_name: None,
            default_backend: None,
        };

        controller.create_ingress_rule(
            "test-ingress".to_string(),
            "default".to_string(),
            spec,
        ).unwrap();

        let route = controller.route_request("example.com", "/api/v1/users");
        assert!(route.is_some());
        
        let route = route.unwrap();
        assert_eq!(route.backend.service.name, "api-service");
        assert_eq!(route.backend.service.port.number, Some(8080));
    }
}