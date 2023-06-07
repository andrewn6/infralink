use hyper::body::to_bytes;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};
use hyper::client::Client;

use std::convert::Infallible;
use std::str;
use colored::*;

pub mod docker;
use docker::utils::DockerClient;

async fn handle_run(image: String) -> Result<hyper::Body, hyper::Error> {
	let registry_service_url = "http://localhost:8083/pull";

	let client = Client::new();

	let req = Request::post(registry_service_url)
		.body(Body::from(image.clone()))
		.expect("request builder");
	let resp = client.request(req).await.unwrap();

	if !resp.status().is_success() {
		return Ok(Body::from("Failed to pull image"));
	}

	let docker_client = DockerClient::new();
	let container_id = docker_client.start_container(&image).await.unwrap();

	Ok(Body::from(format!("Successfully started container with id: {}", container_id)))
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
	let (parts, body) = req.into_parts();

	let response = match (parts.method, parts.uri.path()) {
		(Method::POST, "/run") => {
			let full_body = to_bytes(body).await.unwrap();
			let image = str::from_utf8(&full_body).unwrap().to_string();
			match handle_run(image.to_string()).await {
				Ok(container_id) => Response::new(container_id),
				Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Failed to start container")).unwrap(),
			}
		}
		_ => Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap(),
	};

	Ok(response)
}


async fn run_server() {
	let make_svc = make_service_fn(|_conn| async {
		Ok::<_, Infallible>(service_fn(handle_request))
	});

	let addr = ([127, 0, 0 ,1], 8085).into();
	let server = Server::bind(&addr).serve(make_svc);

	println!("Runner Server listening on {}", addr.to_string().bright_yellow());

	if let Err(e) = server.await {
		eprintln!("server error: {}", e);
	}
}

#[tokio::main] 
async fn main() {
	run_server().await;
}