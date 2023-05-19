use std::sync::atomic::{AtomicUsize, Ordering as OtherOrdering};

use std::sync::mpsc::{self};
use std::sync::mpsc::{Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use lapin::types::FieldTable;
use reqwest::blocking::Client;
use dotenv_codegen::dotenv;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{Notify};
use futures_util::stream::StreamExt as FuturesStreamExt;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use lapin::options::{BasicAckOptions, QueueDeclareOptions, BasicConsumeOptions, BasicPublishOptions};
use lapin::{Channel, Connection, ConnectionProperties, BasicProperties};


const VULTR_API_KEY: &str = dotenv!("VULTR_API_KEY");
const VULTR_API_BASE: &str = "https://api.vultr.com/v2/";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metrics {
	pub cpu: f64,
	pub memory: f64,
	pub disk: f64,
	pub network: f64,
	pub workload: f64,
	pub time: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct WorkerState {
	pub id: usize,
	//pub instance_id: String,
	pub channel: Channel,
	pub notify: Arc<Notify>,
	pub worker_id: usize,
	pub workload: f64,
}

fn create_vultr_instance(worker_id: usize) -> Result<String, Box<dyn std::error::Error>> {
	let client = Client::new();

	let response = client.post(format!("{} instances", VULTR_API_BASE))
		.header("Authorization", format!("Bearer {}", VULTR_API_KEY))
		.json(&json!({
			"region": "ewr",
			"plan": "vc2-6c-16gb",
			"label": format!("worker-{}", worker_id),
			"os_id": 215,
		}))
		.send()?;
	if response.status().is_success() {
		let json: serde_json::Value = response.json()?;
		let instance_id = json["instance"]["id"].as_str().unwrap().to_string();
		info!("Created instance {}", instance_id);
		Ok(instance_id)
	} else {
		error!("Failed to instance: {}", response.text()?);
		Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::Other,
			"Failed to create instance",
		)))
	}
}

fn delete_vultr_instance(instance_id: &str) -> Result<String, Box<dyn std::error::Error>> {
	let client = Client::new();

	let response = client.delete(format!("{}instances/{}", VULTR_API_BASE, instance_id))
		.header("Authorization", format!("Bearer {}", VULTR_API_KEY))
		.send()?;

	if response.status().is_success() {
		info!("Successfully deleted instance with ID: {}", instance_id);
		Ok(instance_id.to_string())
	} else {
		error!("failed to delete instance: {}", response.text()?);
		Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::Other,
			"Failed to delete instance",
		)))
	}
}

async fn scale_down(
    num_workers: &AtomicUsize,
    worker_states: &mut Vec<WorkerState>,
    channel: &Channel,
    metrics_rx: &mpsc::Receiver<Metrics>,
    workload_threshold: f64,
) -> Result<(), Box<dyn std::error::Error>> {
	let mut deleted_ids = Vec::new();

	let metrics = metrics_rx.recv()?;
	
	for worker in worker_states.iter() {
		if metrics.workload < workload_threshold {
			match delete_vultr_instance(&format!("{}", worker.id)) {
				Ok(_) => {
					info!("Deleted instance {}", worker.id);
					num_workers.fetch_sub(1, OtherOrdering::SeqCst);
					deleted_ids.push(worker.id);

					let _ = channel.basic_publish(
						"",
						"worker_deletion",
						BasicPublishOptions::default(),
						format!("worker {} deleted", worker.id).as_bytes(),
						BasicProperties::default(),
					).await?;
				},
				Err(err) => error!("failed to scale down: {}", err),
			}
		}
	}
	*worker_states = worker_states.clone().into_iter().filter(|w| !deleted_ids.contains(&w.id)).collect();
	Ok(())
}

async fn scale_up(
	id: usize,
	rx: &std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Metrics>>>,
	tx: &std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Sender<Metrics>>>,
	notify: Arc<tokio::sync::Notify>,
	channel: &Channel,
	num_workers: &mut usize,
) -> Result<WorkerState, Box<dyn std::error::Error + Send + Sync>> {

	let metrics = {
		let rx = rx.lock().unwrap();
		rx.recv()?
	};

	info!("received metrics: {:?}", metrics);

	match create_vultr_instance(*num_workers) {
		Ok(instance_id) => {
			info!("Sucessfully scaled up by creating a new instance with ID: {}", instance_id);
			*num_workers += 1;

			let worker_state = WorkerState {
				id: *num_workers,
				channel: channel.clone(),
				notify: notify.clone(),
				worker_id: id,
				workload: 0.0,
			};

			let message = format!("Created instance with ID: {}", instance_id);
			let _ = channel.basic_publish(
				"",
				"instance_creation",
				BasicPublishOptions::default(),
				message.as_bytes(),
				BasicProperties::default(),
			).await?;
			let tx_guard = tx.lock().unwrap();
			tx_guard.send(metrics)?;

			Ok(worker_state)
		},
		Err(err) => {
			error!("Failed to scale up: {}", err);
			Err(Box::new(std::io::Error::new(
				std::io::ErrorKind::Other,
				"Failed to scale up",
			)))
		}
	}
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
	// Creates a channel to communciate between threads
	tracing_subscriber::registry()
		.with(fmt::layer())
		.try_init()
		.ok();

	let addr = "amqp://myuser:mypass@localhost:5672/%2f";

	let conn = Connection::connect(addr, ConnectionProperties::default())
		.await
		.unwrap();

	info!("Connected to RabbitMQ");

	let channel = conn.create_channel().await.unwrap();
	let num_workers = AtomicUsize::new(0);

	let (tx, rx) = mpsc::channel::<Metrics>();
	let tx = std::sync::Arc::new(std::sync::Mutex::new(tx));

	let notify = Arc::new(Notify::new());
	let metrics_rx = Arc::new(Mutex::new(rx));
	let mut worker_states: Vec<WorkerState> = Vec::new();

	let queue = channel
		.queue_declare(
			"worker",
			QueueDeclareOptions::default(),
			FieldTable::default(),
		)
		.await
		.unwrap();
	
	let workload_threshold = 0.0;
	let scaling_factor = 2;

	loop {
		let metrics = metrics_rx.lock().expect("Failed to acquire lock").recv().expect("Failed to receive metrics");

		if num_workers.load(OtherOrdering::SeqCst)  < (metrics.workload * scaling_factor as f64).round() as usize {
			let mut num_workers_val = num_workers.load(OtherOrdering::SeqCst);
			if let Ok(worker_state) = scale_up(
				num_workers_val,
				&Arc::new(Mutex::new(rx)),
				&Arc::clone(&tx),
				notify.clone(),
				&channel,
				&mut num_workers_val,
			)
			.await
			{
				worker_states.push(worker_state);

				let consumer_tag = format!("consumer_{}", worker_state.id);
				let consumer = channel.basic_consume(
					queue.name().as_str(),
					&consumer_tag,
					BasicConsumeOptions::default(),
					FieldTable::default(),
				)
				.await
				.unwrap();

				tokio::spawn(async move {
					// Convert to a stream 
					let mut consumer_stream = consumer.into_stream();

					while let Some(result) = consumer_stream.next().await {
						match result {
							Ok(delivery) => {
								delivery
									.ack(BasicAckOptions::default())
									.await
									.expect("ack error");
		
								let data: Metrics = serde_json::from_slice(&delivery.data).unwrap();
								println!("{:?}", data);
							}
							Err(_) => eprintln!("Consumer error: {:?}", result)
						}
					}
				});
			}
		} else if num_workers.load(OtherOrdering::SeqCst) > (metrics.workload * scaling_factor as f64).round() as usize {
			if let Ok(_) = scale_down(
				&num_workers,
				&mut worker_states,
				&channel,
				&rx,
				workload_threshold,
			)
			.await
			{
				worker_states.retain(|worker| worker.workload >= workload_threshold)
			}
		}

		tokio::time::sleep(Duration::from_secs(10)).await;
	}
}