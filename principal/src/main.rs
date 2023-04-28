pub mod providers;
pub mod shared_config;

use dotenv::dotenv;

fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();
}
    
}
