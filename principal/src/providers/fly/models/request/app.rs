use serde::{Deserialize, Serialize};
use dotenv_codegen::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlyApp {
    pub id: String,
    pub name: String,
    pub machine_count: u32,
    pub network: String,
    pub status: String,
    pub hostname: String,
    pub deployed: bool,
    pub suspended: bool,
    pub organization: Organization,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAppRequest {
    pub app_name: String,
    pub org_slug: Option<String>,
    pub network: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAppResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub network: String,
    pub organization: Organization,
}

impl FlyApp {
    pub async fn create(request: CreateAppRequest) -> Result<CreateAppResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .post(&format!("{}/v1/apps", api_hostname))
            .header("Authorization", &format!("Bearer {}", api_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let app: CreateAppResponse = response.json().await?;
            Ok(app)
        } else {
            Err(format!("Failed to create app: {}", response.status()).into())
        }
    }

    pub async fn list() -> Result<Vec<FlyApp>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .get(&format!("{}/v1/apps", api_hostname))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let apps: Vec<FlyApp> = response.json().await?;
            Ok(apps)
        } else {
            Err(format!("Failed to list apps: {}", response.status()).into())
        }
    }

    pub async fn get(app_name: &str) -> Result<FlyApp, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .get(&format!("{}/v1/apps/{}", api_hostname, app_name))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let app: FlyApp = response.json().await?;
            Ok(app)
        } else {
            Err(format!("Failed to get app: {}", response.status()).into())
        }
    }

    pub async fn delete(app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_token = dotenv!("FLY_API_TOKEN");
        let api_hostname = dotenv!("FLY_API_HOSTNAME");
        
        let response = client
            .delete(&format!("{}/v1/apps/{}", api_hostname, app_name))
            .header("Authorization", &format!("Bearer {}", api_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to delete app: {}", response.status()).into())
        }
    }
}