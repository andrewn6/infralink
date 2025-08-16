use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConfig {
    pub region: String,
    pub api_token: String,
    pub api_url: String,
    pub clients: ClientsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientsConfig {
    inner: HashMap<String, ClientConfig>,
}

impl ClientsConfig {
    pub fn new() -> Self {
        let mut inner = HashMap::new();
        inner.insert("http".to_string(), ClientConfig {
            timeout: 30,
            retries: 3,
        });
        inner.insert("vultr".to_string(), ClientConfig {
            timeout: 30,
            retries: 3,
        });
        inner.insert("hetzner".to_string(), ClientConfig {
            timeout: 30,
            retries: 3,
        });
        Self { inner }
    }

    pub fn vultr(&self) -> &ClientConfig {
        self.inner.get("vultr").unwrap_or(self.inner.get("http").unwrap())
    }

    pub fn hetzner(&self) -> &ClientConfig {
        self.inner.get("hetzner").unwrap_or(self.inner.get("http").unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub timeout: u64,
    pub retries: u32,
}

impl ClientConfig {
    pub fn post(&self, url: &str) -> HttpRequestBuilder {
        HttpRequestBuilder::new("POST", url, self.timeout)
    }
    
    pub fn get(&self, url: &str) -> HttpRequestBuilder {
        HttpRequestBuilder::new("GET", url, self.timeout)
    }
    
    pub fn delete(&self, url: String) -> HttpRequestBuilder {
        HttpRequestBuilder::new("DELETE", &url, self.timeout)
    }
}

pub struct HttpRequestBuilder {
    method: String,
    url: String,
    timeout: u64,
    headers: std::collections::HashMap<String, String>,
}

impl HttpRequestBuilder {
    pub fn new(method: &str, url: &str, timeout: u64) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            timeout,
            headers: std::collections::HashMap::new(),
        }
    }
    
    pub fn json<T: serde::Serialize>(&mut self, _body: &T) -> &mut Self {
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self
    }
    
    pub fn bearer_auth(&mut self, token: &str) -> &mut Self {
        self.headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        self
    }
    
    pub fn send(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Mock HTTP request for testing
        Ok(format!("Mock {} response from {}", self.method, self.url))
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            retries: 3,
        }
    }
}

impl Default for SharedConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            api_token: "dummy_token".to_string(),
            api_url: "https://api.example.com".to_string(),
            clients: ClientsConfig::new(),
        }
    }
}