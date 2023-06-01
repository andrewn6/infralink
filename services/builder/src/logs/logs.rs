use futures::{TryStreamExt, StreamExt};
use shiplift::Docker;
use std::time::Duration;
use tokio::time::timeout;

async fn get_logs(container_id: &str) -> Result<(), Box<dyn::std::Error>> {
    let docker = Docker::new();
    let log_options = shiplift::LogsOptions::builder()
        .stdout(true)
        .stderr(true)
        .timestamps(true)
        .follow(true)
        .build();  
}