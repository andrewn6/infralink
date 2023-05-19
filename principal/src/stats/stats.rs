use std::{time::{Duration, Instant}, collections::HashMap};
use tokio::time;
use redis::{AsyncCommands, aio::Connection, Client};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

fn main() {
    println!("hello world")
}

#[derive(Deserialize)]
struct NetdataResponse {
    labels: Vec<String>,
    data: Vec<Vec<f64>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metrics {
	pub cpu: f64,
	pub memory: f64,
	pub disk: f64,
	pub network: f64,
	pub workload: f64,
	pub time: DateTime<Utc>,
}

async fn fetch() -> Result<Metrics> {
    // todo: implement netdata
    Ok(Metrics {
        cpu: 0.0,
        memory: 0.0,
        disk: 0.0,
        network: 0.0,
        workload: 0.0,
        time: Utc::now(),
    })
}

async fn store(metrics: HashMap<String, Metrics>, conn: &mut Connection) -> Result<()> {
    for (key, value) in metrics {
        let serialized_metrics = serde_json::to_string(&value)
            .context("Failed to serialize metrics")?;
        conn.set::<_, _, ()>(key, serialized_metrics).await
            .context("Failed to store metrics")?;
    }
    Ok(())
}