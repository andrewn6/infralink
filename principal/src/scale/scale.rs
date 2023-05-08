use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering as OtherOrdering;

use std::sync::Arc;
use std::sync::mpsc::{self};
use std::thread;
use std::time::{Duration, SystemTime};

use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};

use tokio::sync::Notify;
use tokio::time::{self};
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use lapin::options::{QueueBindOptions, QueueDeclareOptions, BasicAckOptions, BasicGetOptions};
use lapin::{Connection, ConnectionProperties, Channel};

use crate::models::metrics::Metrics;

#[derive(Debug)]
pub struct WorkerState {
	pub id: usize,
	pub channel: Channel,
	pub notify: Arc<Notify>,
}

fn worker(id: usize, rx: mpsc::Receiver<Metrics>, notify: Arc<Notify>) -> Metrics {
	loop {
        let metrics = match rx.recv() {
            Ok(metrics) => metrics,
            Err(_) => {
                error!("Worker {} failed to receive metrics", id);
                continue;
            }
        };

        let sleep_time = Duration::from_millis(rand::random::<u64>() % 5000);
        thread::sleep(sleep_time);

        notify.notify_one();
        return metrics; 
    }
}

async fn spawn_worker(
    channel: lapin::Channel,
    id: usize,
    rx: std::sync::mpsc::Receiver<Metrics>,
	tx: &std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Sender<Metrics>>>,
    notify: Arc<tokio::sync::Notify>,
) -> Result<WorkerState, Box<dyn std::error::Error + Send + Sync>> {
    let cloned_notify = notify.clone();

    let worker_task = tokio::spawn(async move {
        let handle = std::thread::spawn(move || {
            let metrics = worker(id, rx, cloned_notify);
            metrics
        });

        let metrics = handle.join().unwrap();

        info!("Worker{} stopped", id);

        channel
            .basic_ack(0, BasicAckOptions::default())
            .await
            .unwrap();

        tx.send(metrics).unwrap();

        channel
    }); 

    let channel = worker_task.await.unwrap();

    Ok(WorkerState {
        id,
        channel,
        notify,
    })
}

async fn scale_up(
	id: usize,
    rx: mpsc::Receiver<Metrics>,
    tx: &std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Sender<Metrics>>>,
    notify: Arc<tokio::sync::Notify>,
    conn: &Connection,
    channel: &Channel,
    num_workers: &mut usize,
) -> Result<WorkerState, Box<dyn std::error::Error + Send + Sync>> {
    let max_workers = 10;
    if *num_workers < max_workers {
        let worker_id = *num_workers;
        *num_workers += 1;
        info!("Scaling up! creating worker{}", worker_id);

        let queue_name = format!("worker{}", worker_id);
        let queue_declare_options = QueueDeclareOptions::default();
        let queue_bind_options = QueueBindOptions::default();

        channel    
            .queue_declare(
                &queue_name,
                queue_declare_options,
                lapin::types::FieldTable::default(),
            )
            .await?;

        channel
            .queue_bind(
                &queue_name,
                "amq.direct",
                &queue_name,
                queue_bind_options,
                lapin::types::FieldTable::default(),
            )
            .await?;

		let (tx, rx) = mpsc::channel::<Metrics>();

		let notify = Arc::new(tokio::sync::Notify::new());
		let worker_mut = spawn_worker(channel.clone(), worker_id, rx, tx, notify.clone());

		
		tokio::spawn(worker_mut);
        info!("scaled up, worker {} created", worker_id);
    } else  {
        info!("can't scale up more! max num of workers reached");
    }

    Ok(WorkerState {
		id,
		channel: channel.clone(),
		notify: notify.clone(),
	})
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
	
	let queue = channel	
		.queue_declare("worker", QueueDeclareOptions::default(), FieldTable::default())
		.await
		.unwrap();
	let worker_state = spawn_worker(channel, 0, rx, &tx, notify)
		.await
		.unwrap();

		num_workers.fetch_add(1, OtherOrdering::SeqCst);
	
	let mut interval = time::interval(Duration::from_secs(5));

	const MAX_WORKERS: usize = 10;
	let num_workers = AtomicUsize::new(0);

	loop {
		interval.tick().await;
	
		let num_workers_value = num_workers.load(OtherOrdering::SeqCst);
		if num_workers_value >= MAX_WORKERS {
			continue;
		}
	
		let num_workers = AtomicUsize::new(0);
	
		tokio::spawn(async move {
			let worker_state = scale_up(
					num_workers_value,
					rx,
					&tx,
					notify.clone(), 
					&conn,
					&channel.clone(),
					&mut num_workers.load(OtherOrdering::Relaxed),
				)
				.await;
				
				if let Ok(worker_state) = worker_state {
					let mut interval = time::interval(Duration::from_secs(5));

					loop {
						interval.tick().await;

						let metrics = worker_state
							.channel
							.basic_get("worker", BasicGetOptions::default())
							.await
							.unwrap();
						
							if let Ok(Some(delivery)) = worker_state.channel.basic_get("worker", BasicGetOptions::default()).await {
								let metrics: Metrics = serde_json::from_slice(&delivery.data).unwrap();
								tx.lock()
									.unwrap()
									.send(metrics)
									.unwrap_or_else(|err| error!("Failed to send metrics: {}", err));
							}

							let _guard = notify.notified().await;

							break;
					}
				}
		});
		num_workers.fetch_add(1, OtherOrdering::SeqCst);
	}
		
}