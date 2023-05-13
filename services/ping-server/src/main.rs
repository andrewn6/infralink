use ping_rs::send_ping_async;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

pub mod db;

use ping_rs::PingError;
use std::fmt;

#[derive(Debug)]
pub struct Error(PingError);

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Ping error: {:?}", self.0) }
}

impl std::error::Error for Error {}

impl From<PingError> for Error {
	fn from(error: PingError) -> Self { Error(error) }
}

async fn calculate_round_trip(destination_ip: IpAddr) -> Result<u32, Error> {
	let data = [1, 2, 3, 4]; // ping data
	let data_arc = Arc::new(&data[..]);
	let timeout = Duration::from_secs(20);

	// Run this 5 times and take the average
	let mut total_rtt = 0;

	for _ in 0..5 {
		let result = send_ping_async(&destination_ip, timeout, data_arc.clone(), None).await?;
		total_rtt += result.rtt;
	}

	Ok(total_rtt / 5)
}

#[tokio::main]
async fn main() {
	let destination_ip: IpAddr = "8.8.8.8".parse().unwrap();
	match calculate_round_trip(destination_ip).await {
		Ok(avg_rtt) => println!("Average round trip time: {} ms", avg_rtt),
		Err(e) => println!("Error: {}", e),
	}
}
