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
        Self { inner }
    }

    pub fn vultr(&self) -> &ClientConfig {
        self.inner.get("vultr").unwrap_or(&ClientConfig::default())
    }

    pub fn hetzner(&self) -> &ClientConfig {
        self.inner.get("hetzner").unwrap_or(&ClientConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub timeout: u64,
    pub retries: u32,
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