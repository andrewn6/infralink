use colored::Colorize;
use dotenv_codegen::dotenv;
use std::convert::TryInto;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use surge_ping::ping;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

pub mod db;

async fn calculate_round_trip(destination_ip: IpAddr) -> u64 {
	let payload = [0; 8];

	let mut total_rtt = 0;

	for _ in 0..5 {
		let (_packet, duration) = ping(destination_ip, &payload).await.unwrap();
		total_rtt += duration.as_millis();
	}

	(total_rtt / 5).try_into().unwrap()
}

#[tokio::main]
async fn main() {
	let origin_region = dotenv!("REGION");

	let mut connection = db::connection().await.unwrap();

	let ping_map = db::get_airport_ips(&mut connection).await.unwrap();

	let ping_map = Arc::new(Mutex::new(ping_map));

	let ping_map_clone = Arc::clone(&ping_map);

	let update_task = tokio::spawn(async move {
		let mut ping_map_clone = {
			let ping_map_guard = ping_map_clone.lock().await;
			ping_map_guard.clone()
		};

		db::subscribe_to_changes(&mut ping_map_clone).await.unwrap();
	});

	println!("Starting ping server...");

	let ping_task = tokio::spawn(async move {
		loop {
			let ping_map_copy = {
				let ping_map_guard = ping_map.lock().await;
				ping_map_guard.clone()
			};

			for (destination_region, ip) in ping_map_copy.iter() {
				let destination_ip = IpAddr::from_str(ip).unwrap();

				// Calculate the round trip time
				let rtt = calculate_round_trip(destination_ip).await;

				// Store the round trip time in Redis
				db::store_ping(origin_region, destination_region, rtt)
					.await
					.unwrap();

				println!(
					"[{}] updated ping times from {} {} {} [{} milliseconds]",
					chrono::Local::now().to_rfc3339().bright_black(),
					origin_region.bright_cyan(),
					"->".bright_magenta(),
					destination_region.bright_green(),
					rtt.to_string().bright_black()
				);
			}

			time::sleep(Duration::from_millis(5000)).await;
		}
	});

	let _ = tokio::try_join!(update_task, ping_task);
}
