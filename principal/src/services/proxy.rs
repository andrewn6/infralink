use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::services::ingress::{IngressController, RouteMatch};
use crate::services::discovery::ServiceDiscovery;

/// HTTP proxy server for routing ingress traffic
#[derive(Clone)]
pub struct IngressProxy {
    ingress_controller: Arc<IngressController>,
    service_discovery: Arc<ServiceDiscovery>,
    port: u16,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl IngressProxy {
    pub fn new(
        ingress_controller: Arc<IngressController>,
        service_discovery: Arc<ServiceDiscovery>,
        port: u16,
    ) -> Self {
        Self {
            ingress_controller,
            service_discovery,
            port,
        }
    }

    /// Start the proxy server
    pub async fn start(&self) -> Result<(), ProxyError> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        println!("Ingress proxy listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("New connection from: {}", addr);
                    let proxy = self.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = proxy.handle_connection(stream).await {
                            eprintln!("Error handling connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle incoming HTTP connection
    async fn handle_connection(&self, mut stream: TcpStream) -> Result<(), ProxyError> {
        let mut buffer = vec![0; 8192];
        let bytes_read = stream.read(&mut buffer).await?;
        
        if bytes_read == 0 {
            return Ok(());
        }

        let request_data = &buffer[..bytes_read];
        let request = self.parse_http_request(request_data)?;

        // Extract host from headers
        let host = request.headers
            .get("host")
            .or_else(|| request.headers.get("Host"))
            .cloned()
            .unwrap_or_default();

        println!("Routing request: {} {} (Host: {})", request.method, request.path, host);

        // Find matching ingress rule
        let route_match = self.ingress_controller.route_request(&host, &request.path);

        let response = match route_match {
            Some(route) => {
                self.proxy_request(request, route).await?
            }
            None => {
                self.create_404_response()
            }
        };

        // Send response back to client
        let response_bytes = self.format_http_response(&response);
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Parse HTTP request from raw bytes
    fn parse_http_request(&self, data: &[u8]) -> Result<HttpRequest, ProxyError> {
        let request_str = String::from_utf8_lossy(data);
        let lines: Vec<&str> = request_str.lines().collect();

        if lines.is_empty() {
            return Err(ProxyError::InvalidRequest("Empty request".to_string()));
        }

        // Parse request line
        let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line_parts.len() != 3 {
            return Err(ProxyError::InvalidRequest("Invalid request line".to_string()));
        }

        let method = request_line_parts[0].to_string();
        let path = request_line_parts[1].to_string();
        let version = request_line_parts[2].to_string();

        // Parse headers
        let mut headers = HashMap::new();
        let mut body_start = 1;
        
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }

            if let Some(colon_pos) = line.find(':') {
                let header_name = line[..colon_pos].trim().to_lowercase();
                let header_value = line[colon_pos + 1..].trim().to_string();
                headers.insert(header_name, header_value);
            }
        }

        // Extract body (if any)
        let body = if body_start < lines.len() {
            lines[body_start..].join("\n").into_bytes()
        } else {
            Vec::new()
        };

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    /// Proxy request to backend service
    async fn proxy_request(
        &self,
        request: HttpRequest,
        route_match: RouteMatch,
    ) -> Result<HttpResponse, ProxyError> {
        // Resolve service endpoint
        let service_name = &route_match.backend.service.name;
        let port = route_match.backend.service.port.number
            .ok_or_else(|| ProxyError::InvalidBackend("No port specified".to_string()))?;

        // Try to discover service endpoint
        let endpoints = self.service_discovery.get_service_endpoints(service_name);
        
        if endpoints.is_empty() {
            return Ok(self.create_503_response("Service unavailable"));
        }

        // For now, use the first endpoint (could implement load balancing here)
        let endpoint = &endpoints[0];
        let backend_url = format!("{}:{}", endpoint.ip, port);

        println!("Proxying to backend: {}", backend_url);

        // For this mock implementation, return a successful response
        // In a real implementation, this would make an actual HTTP request to the backend
        self.mock_backend_response(&request, &backend_url).await
    }

    /// Mock backend response for demonstration
    async fn mock_backend_response(
        &self,
        request: &HttpRequest,
        backend_url: &str,
    ) -> Result<HttpResponse, ProxyError> {
        // Simulate different responses based on path
        let (status_code, status_text, body) = match request.path.as_str() {
            path if path.starts_with("/api/health") => {
                (200, "OK", r#"{"status": "healthy", "service": "mock-backend"}"#)
            }
            path if path.starts_with("/api/users") => {
                (200, "OK", r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#)
            }
            path if path.starts_with("/api") => {
                (200, "OK", r#"{"message": "API endpoint", "path": ""}"#)
            }
            "/" => {
                (200, "OK", "<!DOCTYPE html><html><head><title>Infralink</title></head><body><h1>Welcome to Infralink</h1><p>Container orchestration platform</p></body></html>")
            }
            _ => {
                (404, "Not Found", r#"{"error": "Endpoint not found"}"#)
            }
        };

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), 
            if request.path == "/" { 
                "text/html".to_string() 
            } else { 
                "application/json".to_string() 
            });
        headers.insert("server".to_string(), "infralink-proxy/1.0".to_string());
        headers.insert("x-backend".to_string(), backend_url.to_string());

        Ok(HttpResponse {
            status_code,
            status_text: status_text.to_string(),
            headers,
            body: body.as_bytes().to_vec(),
        })
    }

    /// Create 404 Not Found response
    fn create_404_response(&self) -> HttpResponse {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("server".to_string(), "infralink-proxy/1.0".to_string());

        HttpResponse {
            status_code: 404,
            status_text: "Not Found".to_string(),
            headers,
            body: r#"{"error": "No ingress rule found for this request"}"#.as_bytes().to_vec(),
        }
    }

    /// Create 503 Service Unavailable response
    fn create_503_response(&self, message: &str) -> HttpResponse {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("server".to_string(), "infralink-proxy/1.0".to_string());

        let body = format!(r#"{{"error": "{}"}}"#, message);

        HttpResponse {
            status_code: 503,
            status_text: "Service Unavailable".to_string(),
            headers,
            body: body.as_bytes().to_vec(),
        }
    }

    /// Format HTTP response as bytes
    fn format_http_response(&self, response: &HttpResponse) -> Vec<u8> {
        let mut result = Vec::new();

        // Status line
        let status_line = format!("HTTP/1.1 {} {}\r\n", response.status_code, response.status_text);
        result.extend_from_slice(status_line.as_bytes());

        // Headers
        for (name, value) in &response.headers {
            let header_line = format!("{}: {}\r\n", name, value);
            result.extend_from_slice(header_line.as_bytes());
        }

        // Content-Length header
        let content_length = format!("Content-Length: {}\r\n", response.body.len());
        result.extend_from_slice(content_length.as_bytes());

        // Empty line to separate headers from body
        result.extend_from_slice(b"\r\n");

        // Body
        result.extend_from_slice(&response.body);

        result
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Invalid backend: {0}")]
    InvalidBackend(String),
    #[error("Backend error: {0}")]
    BackendError(String),
}

/// Ingress proxy configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub port: u16,
    pub read_timeout: std::time::Duration,
    pub write_timeout: std::time::Duration,
    pub max_request_size: usize,
    pub enable_tls: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            read_timeout: std::time::Duration::from_secs(30),
            write_timeout: std::time::Duration::from_secs(30),
            max_request_size: 1024 * 1024, // 1MB
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ingress::*;

    #[tokio::test]
    async fn test_parse_http_request() {
        let ingress_controller = Arc::new(IngressController::new());
        let service_discovery = Arc::new(ServiceDiscovery::new());
        let proxy = IngressProxy::new(ingress_controller, service_discovery, 8080);

        let request_data = b"GET /api/users HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.64.1\r\n\r\n";
        let request = proxy.parse_http_request(request_data).unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/api/users");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.headers.get("host"), Some(&"example.com".to_string()));
    }

    #[test]
    fn test_format_http_response() {
        let ingress_controller = Arc::new(IngressController::new());
        let service_discovery = Arc::new(ServiceDiscovery::new());
        let proxy = IngressProxy::new(ingress_controller, service_discovery, 8080);

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let response = HttpResponse {
            status_code: 200,
            status_text: "OK".to_string(),
            headers,
            body: b"Hello World".to_vec(),
        };

        let formatted = proxy.format_http_response(&response);
        let response_str = String::from_utf8_lossy(&formatted);

        assert!(response_str.contains("HTTP/1.1 200 OK"));
        assert!(response_str.contains("content-type: text/plain"));
        assert!(response_str.contains("Content-Length: 11"));
        assert!(response_str.contains("Hello World"));
    }
}