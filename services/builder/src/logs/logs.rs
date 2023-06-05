use shiplift::Docker;
use shiplift::LogsOptions;
use tokio::sync::broadcast;

use clickhouse_rs::{Block, Pool, types::Decimal};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::config::ClientConfig;

use chrono::prelude::*;
use futures::StreamExt;
use tracing::{error};
use std::str;

pub struct LogMessage {
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub text: String,
}

pub struct LogFilter {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

impl LogFilter {
    pub fn matches(&self, message: &LogMessage) -> bool {
        message.timestamp >= self.start_time && message.timestamp <= self.end_time
    }
}

pub async fn get_logs(container_id: &str, filter: LogFilter, tx: broadcast::Sender<LogMessage>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let docker = Docker::new();

    let container = docker.containers().get(container_id);
    let options = LogsOptions::builder().stdout(true).stderr(true).build();
    let mut logs_stream = container.logs(&options);

    let pool: Pool:new()

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "redpanda:18081")
        .set("message.timeout.ms", "5000")
        .create()?;

    let pool = Pool::new("tcp://clickhouse:9000");

    while let Some(log_result) = logs_stream.next().await {
        match log_result {
            Ok(log_output) => {
                let log_data = str::from_utf8(&log_output)?;
                let parts: Vec<&str> = log_data.splitn(2, ' ').collect();
                let timestamp = parts[0].parse::<DateTime<Utc>>()?;
                let text = parts[1].to_string();
                let message = LogMessage {
                    source: container_id.to_string(),
                    timestamp,
                    text,
                };
                if filter.matches(&message) {
                    let topic = "logs_topic";
                    let record = FutureRecord::to(topic).payload(&format!("{:?}", message)).key("");

                    producer.send(record, 0).await?;
                }

                let mut block = Block::new();
                block.push(("source", message.source));
                block.push(("timestamp", message.timestamp));
                block.push(("text", message.text));

                let mut client = pool.get_handle();
                client.insert("INSERT INTO logs (source, timestamp, text) VALUES", block).await?;
            },
            Err(e) => {
                error!("Error reading logs: {}", e);
            }
        }
    }

    Ok(())
}