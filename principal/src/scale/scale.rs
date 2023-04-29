use std::fs::File;
use std::io::{BufRead, BufReader};

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

use tokio::sync::Notify;
use serde::{Deserialize, Serialize};

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::DefaultConsumer,
};

/* Holds metric data  */
#[derive(Debug, Deserialize, Serialize)]
pub struct Metrics {
    cpu: f64,
    memory: f64,
    disk: f64,
    network: f64,
    time: SystemTime,
}

#[tokio::main(flavor = "current_thread", workers=2)]
fn main() {
    /* Creates a channel to communciate between threads */
    tracing_subscriber::registry()  
        .with(fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let connection = Connection::open(&OpenConnectionArguments::new(
            "localhost",
            5672,
            "guest",
            "guest",
    ))  
    .await
    .unwrap();
    connection
        .register_callback(DefaultConnectionCallback::new())
        .await()
        .unwrap();

    let (tx, rs) = mpsc::channel::<Metrics>();
    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback::new())
        .await
        .unwrap();

    /*  Set thresholds for latency and request rates */
    let request_rate_threshold = 10.0;
    let latency_threshold = 100.0;
    let cpu_threshold = 70.0;
    let memory_threshold = 80.0;
    
    let queue_name = "metrics_queue";
    let consume_callback = move |delivery: amqprs::message::DeliveryResult| {
        match delivery {
            Ok(delivery) => {
                let message = String::from_utf8(delivery.body).unwrap();
                if let Ok(metrics) = serde_json::from_str::<Metrics>(&message) {
                    tx.send(metrics).unwrap();
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    };
    let queue_declare_args = QueueDeclareArguments::default();
    let queue_consume_args = BasicConsumeArguments::default();
    let queue_bind_args = QueueBindArguments::default();
    
    channel 
        .queue_bind(
            queue_name,
            "",
            queue_bind_args,
            None,
        )
        .await
        .unwrap();
    
    let consumer = DefaultConsumer::new(
        consume_callback,
        None
    );

    let _consumer_tag = channel
        .basic_consume(
            queue_name,
            consumer,
            queue_consume_args,
            queue_declare_args,
        )
        .await
        .unwrap();

    /* Spawn a thread to read data from a file and send it over the channel (this will be replaced with a Podman container in the future) */
    thread::spawn(move || {
      let file = File::open("../data/dummy_data.json").unwrap();
      let reader = BufReader::new(file);

      for line in reader.lines() {
        if let Ok(json_str) = line {
          if let Ok(metrics) = serde_json::from_str::<Metrics>(&json_str) {
            let message = amqprs::message::BasicPublishMessage::default()
                .with_body(json_str.as_bytes().to_vec())
                .with_properties(
                    amqprs::message::BasicPublishProperties::default()
                        .with_content_type("application/json".to_string())
                        .with_delivery_mode(amqprs::message::DeliveryMode::Persistent),
                );
            channel.basic_publish(message, "", queue_name).await.unwrap();
                
          }
        }
      }  
    });

    thread::spawn(move || {
        let mut metrics = Vec::<Metrics>::new();
        let mut last_notification = SystemTime::now();
        let notify = Notify::new();

        loop {
            match rx.try_recv() { 
                Ok(metric) => {
                    metrics.push(metric);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    let mut cpu_total = 0.0;
                    let mut memory_total = 0.0;
                    let mut disk_total = 0.0;
                    let mut network_total = 0.0;

                    let mut num_metrics = metrics.len() as f64;

                    for metric in metrics.iter() {
                        cpu_total += metric.cpu;
                        memory_total += metric.memory;
                        disk_total += metric.disk;
                        network_total += metric.network;
                    }

                    if num_metrics > 0.0 {
                        let cpu_avg = cpu_total / num_metrics;
                        let memory_avg = memory_total / num_metrics;
                        let disk_avg = disk_total / num_metrics;
                        let network_avg = network_total / num_metrics;

                        println!("CPU: {} Memory: {} Disk: {} Network: {}", cpu_avg, memory_avg, disk_avg, network_avg);
                    }

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
                    /* Clear metrics after */
                    num_metrics = 0.0;

                    last_notification = SystemTime::now();

                    notify.notify_one();
                }
            }
           
        }

        std::thread::sleep(Duration::from_millis(1000));
        
    });
}