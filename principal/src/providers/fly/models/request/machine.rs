use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use dotenv_codegen::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlyMachine {
    pub id: String,
    pub name: String,
    pub state: String,
    pub region: String,
    pub private_ip: Option<String>,
    pub config: MachineConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineConfig {
    pub image: String,
    pub guest: GuestConfig,
    pub env: Option<HashMap<String, String>>,
    pub services: Option<Vec<Service>>,
    pub init: Option<InitConfig>,
    pub restart: Option<RestartConfig>,
    pub checks: Option<HashMap<String, HealthCheck>>,
    pub mounts: Option<Vec<Mount>>,
    pub auto_destroy: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuestConfig {
    pub cpu_kind: String,  // "shared" or "performance"
    pub cpus: u32,
    pub memory_mb: u32,
    pub gpu_kind: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub ports: Vec<Port>,
    pub protocol: String,  // "tcp" or "udp"
    pub internal_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Port {
    pub port: u16,
    pub handlers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitConfig {
    pub exec: Vec<String>,
    pub entrypoint: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestartConfig {
    pub policy: String,  // "always", "never", "on-failure"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthCheck {
    pub grace_period: Option<String>,
    pub interval: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub port: Option<u16>,
    pub protocol: Option<String>,
    pub timeout: Option<String>,
    pub tls_skip_verify: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub type_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMachineRequest {
    pub config: MachineConfig,
    pub name: Option<String>,
    pub region: Option<String>,
    pub skip_launch: Option<bool>,
    pub skip_service_registration: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMachineResponse {
    pub id: String,
    pub name: String,
    pub state: String,
    pub region: String,
    pub instance_id: String,
    pub private_ip: String,
    pub config: MachineConfig,
    pub created_at: String,
    pub updated_at: String,
}

impl FlyMachine {
    pub async fn create(
        app_name: &str,
        request: CreateMachineRequest,
    ) -> Result<CreateMachineResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .post(&format!("{}/v1/apps/{}/machines", api_hostname, app_name))
            .header("Authorization", &format!("Bearer {}", api_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let machine: CreateMachineResponse = response.json().await?;
            Ok(machine)
        } else {
            Err(format!("Failed to create machine: {}", response.status()).into())
        }
    }

    pub async fn list(app_name: &str) -> Result<Vec<FlyMachine>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .get(&format!("{}/v1/apps/{}/machines", api_hostname, app_name))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let machines: Vec<FlyMachine> = response.json().await?;
            Ok(machines)
        } else {
            Err(format!("Failed to list machines: {}", response.status()).into())
        }
    }

    pub async fn get(
        app_name: &str,
        machine_id: &str,
    ) -> Result<FlyMachine, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .get(&format!("{}/v1/apps/{}/machines/{}", api_hostname, app_name, machine_id))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let machine: FlyMachine = response.json().await?;
            Ok(machine)
        } else {
            Err(format!("Failed to get machine: {}", response.status()).into())
        }
    }

    pub async fn start(
        app_name: &str,
        machine_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .post(&format!("{}/v1/apps/{}/machines/{}/start", api_hostname, app_name, machine_id))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to start machine: {}", response.status()).into())
        }
    }

    pub async fn stop(
        app_name: &str,
        machine_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .post(&format!("{}/v1/apps/{}/machines/{}/stop", api_hostname, app_name, machine_id))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to stop machine: {}", response.status()).into())
        }
    }

    pub async fn delete(
        app_name: &str,
        machine_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .delete(&format!("{}/v1/apps/{}/machines/{}", api_hostname, app_name, machine_id))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to delete machine: {}", response.status()).into())
        }
    }
}