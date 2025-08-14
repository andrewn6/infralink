use serde::{Deserialize, Serialize};
use dotenv_codegen::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlyVolume {
    pub id: String,
    pub name: String,
    pub size_gb: u32,
    pub region: String,
    pub state: String,
    pub attached_machine_id: Option<String>,
    pub attached_alloc_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVolumeRequest {
    pub name: String,
    pub size_gb: u32,
    pub region: String,
    pub snapshot_id: Option<String>,
    pub encrypted: Option<bool>,
    pub require_unique_zone: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVolumeResponse {
    pub id: String,
    pub name: String,
    pub size_gb: u32,
    pub region: String,
    pub state: String,
    pub created_at: String,
}

impl FlyVolume {
    pub async fn create(
        app_name: &str,
        request: CreateVolumeRequest,
    ) -> Result<CreateVolumeResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .post(&format!("{}/v1/apps/{}/volumes", api_hostname, app_name))
            .header("Authorization", &format!("Bearer {}", api_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let volume: CreateVolumeResponse = response.json().await?;
            Ok(volume)
        } else {
            Err(format!("Failed to create volume: {}", response.status()).into())
        }
    }

    pub async fn list(app_name: &str) -> Result<Vec<FlyVolume>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .get(&format!("{}/v1/apps/{}/volumes", api_hostname, app_name))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let volumes: Vec<FlyVolume> = response.json().await?;
            Ok(volumes)
        } else {
            Err(format!("Failed to list volumes: {}", response.status()).into())
        }
    }

    pub async fn get(
        app_name: &str,
        volume_id: &str,
    ) -> Result<FlyVolume, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .get(&format!("{}/v1/apps/{}/volumes/{}", api_hostname, app_name, volume_id))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let volume: FlyVolume = response.json().await?;
            Ok(volume)
        } else {
            Err(format!("Failed to get volume: {}", response.status()).into())
        }
    }

    pub async fn delete(
        app_name: &str,
        volume_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .delete(&format!("{}/v1/apps/{}/volumes/{}", api_hostname, app_name, volume_id))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to delete volume: {}", response.status()).into())
        }
    }

    pub async fn extend(
        app_name: &str,
        volume_id: &str,
        size_gb: u32,
    ) -> Result<FlyVolume, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let request_body = serde_json::json!({
            "size_gb": size_gb
        });
        
        let response = client
            .put(&format!("{}/v1/apps/{}/volumes/{}/extend", api_hostname, app_name, volume_id))
            .header("Authorization", &format!("Bearer {}", api_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let volume: FlyVolume = response.json().await?;
            Ok(volume)
        } else {
            Err(format!("Failed to extend volume: {}", response.status()).into())
        }
    }
}