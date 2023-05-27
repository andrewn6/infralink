use std::collections::HashMap;

use dotenv_codegen::dotenv;
use futures::stream::StreamExt;
use indexmap::IndexMap;
use redis::aio::Connection;
use redis::{AsyncCommands, RedisResult};

use crate::utils;

pub async fn connection() -> RedisResult<Connection> {
	let client = redis::Client::open(dotenv!("MASTER_REDIS_CONNECTION_URL"))?;

	let connection = client.get_async_connection().await?;

	Ok(connection)
}

pub async fn get_ping_map(connection: &mut Connection) -> RedisResult<HashMap<String, String>> {
	let mut result: HashMap<String, String> = HashMap::new();

	let keys: Vec<String> = connection.hkeys("ping").await?;

	for key in keys {
		let value: String = connection.hget("ping", &key).await?;
		result.insert(key, value);
	}

	Ok(result)
}

pub async fn subscribe_to_changes(
	state: &mut (HashMap<String, String>, IndexMap<String, String>),
) -> RedisResult<()> {
	let pubsub_connection = connection().await?; // Create a new connection for pubsub.

	let mut pubsub = pubsub_connection.into_pubsub();

	pubsub.subscribe("__keyspace@0__:ping").await?;
	println!("Subscribed to changes in 'ping'");

	let mut connection: Connection = connection().await?;

	loop {
		let msg = pubsub.on_message().next().await;
		match msg {
			Some(msg) => {
				let payload: String = msg.get_payload()?;
				println!("Changes detected in 'ping'");

				if payload == "set" {
					// If the key has been set, update the ping_map and routing_table
					let new_ping_map = get_ping_map(&mut connection).await?;
					let new_routing_table = utils::build_routing_table(new_ping_map.clone());

					*state = (new_ping_map, new_routing_table);
				}
			}
			None => return Ok(()),
		}
	}
}
