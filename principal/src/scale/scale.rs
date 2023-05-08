use std::fs::File;
use std::io::{BufRead, BufReader};

use std::sync::mpsc::{self};
use std::thread;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use tokio::sync::Notify;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use lapin::options::{BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions};
use lapin::{Connection, ConnectionProperties};

use crate::models::metrics::Metrics;

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
	// Creates a channel to communciate between threads
	tracing_subscriber::registry()
		.with(fmt::layer())
		.try_init()
		.ok();

	let addr = "amqp://myuser:mypass@localhost:5672/%2f";

	let conn = Connection::connect(&addr, ConnectionProperties::default())
		.await
		.unwrap();

	info!("Connected to RabbitMQ");

	let (tx, rs) = mpsc::channel::<Metrics>();
	let channel = conn.create_channel().await.unwrap();

	// Set thresholds for latency and request rates
	let request_rate_threshold = 10.0;
	let latency_threshold = 100.0;
	let cpu_threshold = 70.0;
	let memory_threshold = 80.0;

	let queue_name = "metrics_queue".to_string();
	let queue_declare_options = QueueDeclareOptions::default();
	let queue_consume_options = BasicConsumeOptions::default();
	let queue_bind_options = QueueBindOptions::default();

	channel
		.queue_declare(
			&queue_name,
			queue_declare_options,
			lapin::types::FieldTable::default(),
		)
		.await
		.unwrap();

	channel
		.queue_bind(
			&queue_name,
			"amq.direct", // use the default exchange
			&queue_name,  // set the routing key to the queue's name
			queue_bind_options,
			lapin::types::FieldTable::default(),
		)
		.await
		.unwrap();

	let mut consumer = channel
		.basic_consume(
			&queue_name,
			"",
			queue_consume_options,
			lapin::types::FieldTable::default(),
		)
		.await
		.unwrap();

	// Spawn a thread to read data from a file and send it over the channel (this will be replaced with a Podman container in the future)
	thread::spawn(move || {
		let file = File::open("../data/dummy_data.json").unwrap();
		let reader = BufReader::new(file);

		for line in reader.lines() {
			if let Ok(json_str) = line {
				if let Ok(metrics) = serde_json::from_str::<Metrics>(&json_str) {
					tx.send(metrics).unwrap();
				}
			}
		}
	});

	thread::spawn(move || {
		let mut metrics = Vec::<Metrics>::new();
		let mut last_notification = SystemTime::now();
		let notify = Notify::new();

		loop {
			match rs.try_recv() {
				Ok(metric) => {
					metrics.push(metric);
				}
				Err(mpsc::TryRecvError::Empty) => {
					let mut cpu_total = 0.0;
					let mut memory_total = 0.0;
					let mut disk_total = 0.0;
					let mut network_total = 0.0;

					let num_metrics = metrics.len() as f64;

					for metric in metrics.iter() {
						cpu_total += metric.cpu;
						memory_total += metric.memory;
						disk_total += metric.disk;
						network_total += metric.network;
					}

					let cpu_avg = cpu_total / num_metrics;
					let memory_avg = memory_total / num_metrics;
					let disk_avg = disk_total / num_metrics;
					let network_avg = network_total / num_metrics;
					let network_threshold = 50.0;

					if cpu_avg > cpu_threshold {
						println!("CPU usage is above threshold of {}$", cpu_threshold);
					}

					if memory_avg > memory_threshold {
						println!("Memory usage is above threshold of {}$", memory_threshold);
					}

					if network_avg > network_threshold {
						println!("Network usage is above threshold of {}$", network_threshold);
					}

					let elapsed_time = last_notification.elapsed().unwrap().as_millis() as f64;
					if elapsed_time > latency_threshold {
						println!("Latency is above threshold of {}ms", latency_threshold);
					}

					metrics.clear();
					last_notification = SystemTime::now();

					notify.notify_one();
				}
				Err(mpsc::TryRecvError::Disconnected) => {
					error!("The channel is disconnected, stopping metrics collection");
					break;
				}
			}
			std::thread::sleep(Duration::from_millis(1000));
		}
	});
}
