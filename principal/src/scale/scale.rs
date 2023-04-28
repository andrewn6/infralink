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
            "user",
            "bitnami",
    ))  
    .await
    .unwrap();
    connection
        .register_callback(DefaultChannelCallback::new())
        .await()
        .unwrap();

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
    let queue_declare_arguments = QueueDeclareArguments::default();
    channel.queue_declare(queue_name, queue_declare_arguments).await.unwrap();

    /* Spawn a thread to read data from a file and send it over the channel (this will be replaced with a Podman container in the future) */
    thread::spawn(move || {
      let file = File::open("../data/dummy_data.json").unwrap();
      let reader = BufReader::new(file);

      for line in reader.lines() {
        if let Ok(json_str) = line {
          if let Ok(metrics) = serde_json::from_str::<Metrics>(&json_str) {
            let message = amqprs::message::BasicPublishMessage::default()
                .with_body(json_str.as_bytes().to_vec());
            let queue_binds_args = QueueBindArguments::default();
            channel 
                .basic_publish(
                    "",
                    queue_name,
                    message,
                    queue_binds_args,
                    None,
                )
                .unwrap();
          }
        }
      }  
    });

    let mut replica_count = 1;

    loop {
        let mut request_count = 0;
        let mut latency_sum = 0.0;
        let mut metrics_count = 0;
        let mut cpu_sum = 0.0;
        let mut memory_sum = 0.0;

        for metrics in rx.try_iter() {
            if SystemTime::now().duration_since(metrics.time).unwrap().as_secs() > 60 {
                break;
            }
            request_count += 1;
            latency_sum += metrics.network;
            metrics_count += 1;
            cpu_sum += metrics.cpu;
            memory_sum += metrics.memory;
        }

        if request_count > 0 {
            let request_rate = request_count as f64 / 60.0;
            let latency = latency_sum / metrics_count as f64;
            let cpu = cpu_sum / metrics_count as f64;
            let memory = memory_sum / metrics_count as f64;

            println!("request_rate: {}", request_rate);
            println!("latency: {}", latency);
            println!("cpu usage: {}", latency);
            println!("memory usage: {}", latency);
            
            /* Scale up if request wate/latency exceeds the thresholds */
            if request_rate >  request_rate_threshold 

                && latency < latency_threshold
                && cpu_usage > cpu_threshold
                && memory_usage > memory_threshold
                
            {
                replica_count += 1;
            } else if replica_count > 1
                && (request_count < request_rate_threshold
                    || latency > latency_threshold
                    || cpu_usage < cpu_threshold
                    || memory_usage < memory_threshold)
            
            {
                replica_count -= 1;
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
}