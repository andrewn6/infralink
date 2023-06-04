use std::collections::HashMap;

use dotenv_codegen::dotenv;
use futures::stream::StreamExt;
use redis::aio::Connection;
use redis::{AsyncCommands, RedisResult};

pub async fn connection() -> RedisResult<Connection> {
	let client = redis::Client::open(dotenv!("MASTER_REDIS_CONNECTION_URL"))?;

	let connection = client.get_async_connection().await?;

	Ok(connection)
}

pub async fn subscribe_to_changes(ping_map: &mut HashMap<String, String>) -> RedisResult<()> {
	let pubsub_connection = connection().await?; // Create a new connection for pubsub.

	let mut pubsub = pubsub_connection.into_pubsub();

	pubsub.subscribe("__keyspace@0__:airports").await?;
	println!("Subscribed to changes in 'airports'");

	let mut connection = connection().await?;

	loop {
		let msg = pubsub.on_message().next().await;
		match msg {
			Some(msg) => {
				let payload: String = msg.get_payload()?;

				if payload == "set" {
					// If the key has been set, update the ping_map
					let new_ping_map = get_airport_ips(&mut connection).await?;
					*ping_map = new_ping_map;
				}
			}
			None => return Ok(()),
		}
	}
}

pub async fn store_ping(
	origin_region: &str,
	destination_region: &str,
	rtt: u64,
) -> RedisResult<()> {
	let mut connection = connection().await?;

    let key = format!("ping:{}", origin_region);

    let _: () = connection.zadd(&key, destination_region, rtt as f64).await?;

	Ok(())
}

pub async fn set_airport_ip(
	connection: &mut Connection,
	airport_code: &str,
	ip: &str,
) -> RedisResult<()> {
	connection.hset("airports", airport_code, ip).await?;

	Ok(())
}

pub async fn get_airport_ips(connection: &mut Connection) -> RedisResult<HashMap<String, String>> {
	let mut result: HashMap<String, String> = HashMap::new();

	let keys: Vec<String> = connection.hkeys("airports").await?;

	for key in keys {
		println!("Getting IP for airport: {}", key);
		let value: String = connection.hget("airports", &key).await?;
		result.insert(key, value);
	}

	Ok(result)
}
