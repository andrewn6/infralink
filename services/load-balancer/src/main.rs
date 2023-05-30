use std::sync::Arc;

use tokio::sync::Mutex;

pub mod db;
pub mod utils;

#[tokio::main]
async fn main() {
	let mut connection = db::connection().await.unwrap();

	// fetch the current state of the ping table from the database
	let ping_map = db::get_ping_map(&mut connection).await.unwrap();

	let routing_table = utils::build_routing_table(ping_map.clone());
	let shared_state = Arc::new(Mutex::new((ping_map, routing_table)));

	// You don't need to clone shared_state anymore.
	let shared_state_clone = Arc::clone(&shared_state);
	tokio::spawn(async move {
		db::subscribe_to_changes(shared_state_clone).await.unwrap();
	});

	loop {
		let (ping_map, routing_table) = {
			let state_guard = shared_state.lock().await;
			let state = state_guard.clone();
			state
		};

		println!("Ping map: {:?}", ping_map);
		println!("Routing table: {:?}", routing_table);

		tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
	}
}
