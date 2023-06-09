use hyper::body::to_bytes;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};
use hyper::client::Client;

use std::convert::Infallible;
use std::str;
use std::time::Duration;
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
				Ok(container_id) => {
					let container_id_bytes = hyper::body::to_bytes(container_id)
						.await
						.unwrap()
						.to_vec();

					let container_id_str = String::from_utf8_lossy(&container_id_bytes).to_string();
					let docker_client = DockerClient::new();

					let response_container_id = container_id_str.clone();

					tokio::spawn(async move {
						tokio::time::sleep(Duration::from_secs(60)).await;
						docker_client.stop_container(&response_container_id).await.unwrap();
					});

					let response_body = hyper::Body::from(container_id_str);

					Response::new(response_body)
				},
				Err(e) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(format!("Error: {}", e))).unwrap(),
			}
		}
		(Method::GET, "/status") => {
			let container_id = parts.uri.query().unwrap_or("");
			let docker_client = DockerClient::new();

			match docker_client.get_container_status(container_id).await {
				Ok(details) => {
					let status = format!("Container Status: {:?}", details.state.status);
					Response::new(Body::from(status))
				},
				Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Error getting container status")).unwrap(),
			}
		}
		(Method::POST, "/stop") => {
			let full_body = to_bytes(body).await.unwrap();
			let container_id = String::from_utf8_lossy(&full_body).to_string();
			let docker_client = DockerClient::new();
			match docker_client.stop_container(&container_id).await {
				Ok(_) => Response::new(Body::from("Successfully stopped container")),
				Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Error stopping run")).unwrap(),
			}
		}
		_ => Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Not Found")).unwrap(),
		/* Needs a fix... 
		(Method::GET, "/logs") => {
			let container_id = parts.uri.query().unwrap_or("");
			let docker_client = DockerClient::new();
			match docker_client.stream_logs(container_id).await {
				Ok(logs) => {
					let logs_vec = logs;
					let logs_body: hyper::Body = Body::from(logs);
					Response::new(logs_body)
				},
				Err(_) => Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(Body::from("Error getting logs"))
					.unwrap(),
			}
		}
		*/
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