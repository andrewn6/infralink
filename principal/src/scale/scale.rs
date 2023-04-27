use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

/* Holds metric data  */
#[derive(Debug)]
pub struct Metrics {
    cpu: f64,
    memory: f64,
    disk: f64,
    network: f64,
    time: SystemTime,
}

fn main() {
    /* Creates a channel to communciate between threads */
    let (tx, rs) = mpsc::channel();

    /*  Set thresholds for latency and request rates */
    let request_rate_threshold = 10.0;
    let latency_threshold = 100.0;

    /* Spawn a thread to read data from a file and send it over the channel (this will be replaced with a Podman container in the future) */
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

    let mut replica_count = 1;

    loop {
        let mut request_count = 0;
        let mut latency_sum = 0.0;
        let mut metrics_count = 0;

        for metrics in rx.try_iter() {
            if SystemTime::now().duration_since(metrics.time).unwrap().as_secs() > 60 {
                break;
            }
            request_count += 1;
            latency_sum += metrics.network;
            metrics_count += 1;
        }

        if request_count > 0 {
            let request_rate = request_count as f64 / 60.0;
            let latency = latency_sum / metrics_count as f64;

            println!("request_rate: {}", request_rate);
            println!("latency: {}", latency);
            
            /* Scale up if request wate/latency exceeds the thresholds */
            if request_rate > request_rate_threshold && latency < latency_threshold {
                replica_count += 1;
            /*/ Scale down if request rate/latency is below the thresholds */
            } else if replica_count > 1 {
                replica_count -= 1;
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
}