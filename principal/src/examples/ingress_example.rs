use std::sync::Arc;
use crate::services::ingress::*;
use crate::services::proxy::{IngressProxy, ProxyConfig};
use crate::services::discovery::ServiceDiscovery;

/// Example of setting up ingress rules and running the proxy server
pub async fn run_ingress_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Infralink Ingress Controller Example ===");

    // Initialize services
    let ingress_controller = Arc::new(IngressController::new());
    let mut service_discovery = ServiceDiscovery::new();

    // Create example ingress rules
    setup_example_ingress_rules(&ingress_controller).await?;

    // Register some example services
    setup_example_services(&mut service_discovery).await?;

    let service_discovery = Arc::new(service_discovery);

    // Create and start the proxy server
    let proxy = IngressProxy::new(
        ingress_controller.clone(),
        service_discovery.clone(),
        8080,
    );

    println!("\nIngress proxy server starting on port 8080...");
    println!("Try these URLs:");
    println!("  - http://api.example.com:8080/users");
    println!("  - http://web.example.com:8080/");
    println!("  - http://admin.example.com:8080/dashboard");
    println!("  - http://localhost:8080/health");

    // This would block and run the server
    // proxy.start().await?;

    // For demonstration, just show the configuration
    demonstrate_ingress_routing(&ingress_controller).await?;

    Ok(())
}

async fn setup_example_ingress_rules(
    controller: &IngressController,
) -> Result<(), IngressError> {
    println!("\n1. Creating ingress rules...");

    // API ingress rule
    let api_spec = IngressSpec {
        rules: vec![IngressRuleSpec {
            host: Some("api.example.com".to_string()),
            http: Some(HttpIngressRuleValue {
                paths: vec![
                    HttpIngressPath {
                        path: "/users".to_string(),
                        path_type: PathType::Prefix,
                        backend: IngressBackend {
                            service: IngressServiceBackend {
                                name: "user-service".to_string(),
                                port: ServiceBackendPort {
                                    number: Some(3000),
                                    name: None,
                                },
                            },
                        },
                    },
                    HttpIngressPath {
                        path: "/auth".to_string(),
                        path_type: PathType::Prefix,
                        backend: IngressBackend {
                            service: IngressServiceBackend {
                                name: "auth-service".to_string(),
                                port: ServiceBackendPort {
                                    number: Some(3001),
                                    name: None,
                                },
                            },
                        },
                    },
                ],
            }),
        }],
        tls: vec![IngressTls {
            hosts: vec!["api.example.com".to_string()],
            secret_name: "api-tls-cert".to_string(),
        }],
        ingress_class_name: Some("infralink".to_string()),
        default_backend: None,
    };

    let api_rule = controller.create_ingress_rule(
        "api-ingress".to_string(),
        "production".to_string(),
        api_spec,
    )?;
    println!("✓ Created API ingress rule: {}", api_rule.id);

    // Web frontend ingress rule
    let web_spec = IngressSpec {
        rules: vec![IngressRuleSpec {
            host: Some("web.example.com".to_string()),
            http: Some(HttpIngressRuleValue {
                paths: vec![HttpIngressPath {
                    path: "/".to_string(),
                    path_type: PathType::Prefix,
                    backend: IngressBackend {
                        service: IngressServiceBackend {
                            name: "frontend-service".to_string(),
                            port: ServiceBackendPort {
                                number: Some(80),
                                name: None,
                            },
                        },
                    },
                }],
            }),
        }],
        tls: vec![IngressTls {
            hosts: vec!["web.example.com".to_string()],
            secret_name: "web-tls-cert".to_string(),
        }],
        ingress_class_name: Some("infralink".to_string()),
        default_backend: None,
    };

    let web_rule = controller.create_ingress_rule(
        "web-ingress".to_string(),
        "production".to_string(),
        web_spec,
    )?;
    println!("✓ Created web ingress rule: {}", web_rule.id);

    // Admin panel ingress rule
    let admin_spec = IngressSpec {
        rules: vec![IngressRuleSpec {
            host: Some("admin.example.com".to_string()),
            http: Some(HttpIngressRuleValue {
                paths: vec![HttpIngressPath {
                    path: "/dashboard".to_string(),
                    path_type: PathType::Prefix,
                    backend: IngressBackend {
                        service: IngressServiceBackend {
                            name: "admin-service".to_string(),
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
        ingress_class_name: Some("infralink".to_string()),
        default_backend: None,
    };

    let admin_rule = controller.create_ingress_rule(
        "admin-ingress".to_string(),
        "production".to_string(),
        admin_spec,
    )?;
    println!("✓ Created admin ingress rule: {}", admin_rule.id);

    // Default/catchall ingress rule
    let default_spec = IngressSpec {
        rules: vec![IngressRuleSpec {
            host: None, // Matches any host
            http: Some(HttpIngressRuleValue {
                paths: vec![HttpIngressPath {
                    path: "/health".to_string(),
                    path_type: PathType::Exact,
                    backend: IngressBackend {
                        service: IngressServiceBackend {
                            name: "health-service".to_string(),
                            port: ServiceBackendPort {
                                number: Some(8000),
                                name: None,
                            },
                        },
                    },
                }],
            }),
        }],
        tls: vec![],
        ingress_class_name: Some("infralink".to_string()),
        default_backend: Some(IngressBackend {
            service: IngressServiceBackend {
                name: "default-service".to_string(),
                port: ServiceBackendPort {
                    number: Some(80),
                    name: None,
                },
            },
        }),
    };

    let default_rule = controller.create_ingress_rule(
        "default-ingress".to_string(),
        "production".to_string(),
        default_spec,
    )?;
    println!("✓ Created default ingress rule: {}", default_rule.id);

    Ok(())
}

async fn setup_example_services(
    discovery: &mut ServiceDiscovery,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Registering backend services...");

    // Register services that the ingress rules point to
    let services = vec![
        ("user-service", "127.0.0.1", 3000),
        ("auth-service", "127.0.0.1", 3001),
        ("frontend-service", "127.0.0.1", 80),
        ("admin-service", "127.0.0.1", 8080),
        ("health-service", "127.0.0.1", 8000),
        ("default-service", "127.0.0.1", 80),
    ];

    for (name, ip, port) in services {
        discovery.register_simple_service(
            name.to_string(),
            ip.to_string(),
            port,
            HashMap::new(),
        );
        println!("✓ Registered service: {} at {}:{}", name, ip, port);
    }

    Ok(())
}

async fn demonstrate_ingress_routing(
    controller: &IngressController,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Testing ingress routing...");

    let test_requests = vec![
        ("api.example.com", "/users/123"),
        ("api.example.com", "/auth/login"),
        ("web.example.com", "/"),
        ("web.example.com", "/about"),
        ("admin.example.com", "/dashboard"),
        ("localhost", "/health"),
        ("unknown.com", "/some-path"),
    ];

    for (host, path) in test_requests {
        match controller.route_request(host, path) {
            Some(route_match) => {
                println!("✓ {} {} -> {} (port: {:?})", 
                    host, 
                    path, 
                    route_match.backend.service.name,
                    route_match.backend.service.port.number
                );
            }
            None => {
                println!("✗ {} {} -> No route found", host, path);
            }
        }
    }

    println!("\n4. Ingress health summary:");
    let health = controller.get_health_summary();
    println!("   Total rules: {}", health.total_rules);
    println!("   Active rules: {}", health.active_rules);
    println!("   Total certificates: {}", health.total_certificates);
    println!("   Expired certificates: {}", health.expired_certificates);

    println!("\n5. All ingress rules:");
    for rule in controller.list_ingress_rules() {
        println!("   Rule: {} (namespace: {})", rule.name, rule.namespace);
        for rule_spec in &rule.spec.rules {
            let host = rule_spec.host.as_deref().unwrap_or("*");
            println!("     Host: {}", host);
            
            if let Some(http) = &rule_spec.http {
                for path in &http.paths {
                    println!("       Path: {} -> {}", 
                        path.path, 
                        path.backend.service.name
                    );
                }
            }
        }
    }

    Ok(())
}

use std::collections::HashMap;