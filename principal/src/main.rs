pub mod providers;
pub mod shared_config;

use dotenv::dotenv;

pub mod db;
pub mod scale;
pub mod stats;

fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();
}