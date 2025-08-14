use juniper::{EmptyMutation, FieldResult, GraphQLObject};
use dotenv::dotenv;

pub mod providers;
pub mod shared_config;
pub mod db;
pub mod scale;
pub mod services;

#[derive(GraphQLObject)]
struct Status {
    message: String,
}

struct Query;

#[juniper::graphql_object]
impl Query {
    fn status() -> FieldResult<Status> {
        Ok(Status {
            message: "Infralink is running".to_string(),
        })
    }
}

#[tokio::main]
async fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();
    println!("Principal service started");
}
