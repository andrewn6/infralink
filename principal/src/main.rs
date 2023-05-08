pub mod models;
pub mod providers;
pub mod services;
pub mod shared_config;

use dotenv::dotenv;
pub mod scale;
use crate::scale::scale::main as scaler;

fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();
	// scaler();
}
