pub mod db;

use dotenv::dotenv;

fn main() {
	dotenv().unwrap();
	
	println!("Hello, world!");
}