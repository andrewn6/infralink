pub mod providers;
pub mod shared_config;
pub mod scale;
use dotenv::dotenv;

fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();
}