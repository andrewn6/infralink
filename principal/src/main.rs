use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use juniper::http::playground::playground_source;
use juniper::http::GraphQLRequest;

pub mod providers;
pub mod shared_config;

use dotenv::dotenv;

pub mod db;
pub mod scale;

async fn graphql_handler(
	schema: juniper::RootNode<'static, Query, EmptyMutation<()>>,
	req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
	let ctx = ();
	let query = match juniper_hyper::graphql_request_from_hyper(&req).await {
		Ok(value) => value,
		Err(_) => return Ok(Response::builder().status(400).body(Body::empty())?),
	};

	let response = query.execute(&schema, &ctx).await;
	let body = serde_json::to_string(&response)?;
	Ok(Response::builder()
		.header("Content-Type", "application/json")
		.body(Body::from(body))?)
}

#[tokio::main]
async fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();
}
