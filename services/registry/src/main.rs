use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use serde::Deserialize;
use shiplift::{Docker, PullOptions};
use futures_util::StreamExt;
use tokio::sync::Semaphore;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use colored::*;

#[derive(Deserialize)]
struct ImageData {
	registry_url: String,
	image_name: String,
	image_tag: String,
}

async fn handle_push(
	mut req: Request<Body>,
	docker: Arc<Docker>,
	semaphore: Arc<Semaphore>,
) -> Result<Response<Body>, hyper::Error> {
	let permit = semaphore.acquire().await.unwrap();

	let whole_body = to_bytes(req.body_mut()).await?;
	let image_data: Result<ImageData, _> = serde_json::from_slice(&whole_body);

	let image_data = match image_data {
		Ok(data) => data,
		Err(e) => {
			eprintln!("Failed to parse request: {}", e);
			return Ok(Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(Body::from("Failed to parse request"))
				.unwrap());
		}
	};

	/* Input validation */
	if image_data.image_name.is_empty() || image_data.image_tag.is_empty() {
		return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(Body::from("Image name or tag is empty"))
			.unwrap());
	}	

	if image_data.image_name.is_empty() || image_data.image_tag.is_empty() {
		return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(Body::from("Image name or tag is empty"))
			.unwrap());
	}

	// Update this to not be hard coded
	let image = format!(
		"{}/{}:{}",
		image_data.registry_url, image_data.image_name, image_data.image_tag
	);
	let pull_options = PullOptions::builder().image(&image).build();
	let mut stream = docker.images().pull(&pull_options);

	while let Some(result) = stream.next().await {
		match result {
			Ok(output) => println!("Pull event: {:?}", output),
			Err(err) => eprintln!("error: {}", err),
		}
	}

	let output = Command::new("docker")
		.arg("push")
		.arg(image)
		.output()
		.expect("Failed to execute process");

	drop(permit);

	if output.status.success() {
		Ok(Response::new(Body::from("Image pushed successfully!")))
	} else {
		let error_message = String::from_utf8_lossy(&output.stderr).into_owned();
		eprintln!("error: {}", error_message);
		Ok(Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(Body::from(error_message))
			.unwrap())
	}
}

async fn handle_pull(
	mut req: Request<Body>,
	docker: Arc<Docker>,
	semaphore: Arc<Semaphore>,
) -> Result<Response<Body>, hyper::Error> {
	let whole_body = to_bytes(req.body_mut()).await?;

	let image_data: Result<ImageData, _> = serde_json::from_slice(&whole_body);

	let image_data = match image_data {
		Ok(data) => data,
		Err(e) => {
			eprintln!("Failed to parse request: {}", e);
			return Ok(Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(Body::from("Failed to parse request"))
				.unwrap());
		}
	};

	if image_data.image_name.is_empty() || image_data.image_tag.is_empty() {
		return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(Body::from("Image name or tag is empty"))
			.unwrap());
	}

	let image = format!(
		"{}/{}:{}",
		image_data.registry_url, image_data.image_name, image_data.image_tag
	);
	let pull_options = shiplift::PullOptions::builder().image(&image).build();
	let mut stream = docker.images().pull(&pull_options);

	while let Some(result) = stream.next().await {
		match result {
			Ok(output) => println!("{:?}", output),
			Err(err) => println!("Error: {}", err),
		}
	}

	Ok(Response::new(Body::from("Image pulled successfully")))
}

async fn handle_request(
	req: Request<Body>,
	docker: Arc<Docker>,
	semaphore: tokio::sync::Semaphore,
) -> Result<Response<Body>, hyper::Error> {
	match (req.method(), req.uri().path()) {
		(&Method::POST, "/push") => handle_push(req, docker, semaphore.into()).await,
		(&Method::GET, "/pull") => handle_pull(req, docker, semaphore.into()).await,
		_ => Ok(Response::builder()
			.status(StatusCode::NOT_FOUND)
			.body(Body::empty())
			.unwrap()),
	}
}

/*
fn service_fn_wrapper(docker: Docker) -> impl Fn(Request<Body>) -> futures_util::future::Ready<Result<Response<Body>, hyper::Error>> {
	move |req| futures_util::future::ready(handle_request(req, docker.clone()))
}
*/

#[tokio::main]
async fn main() {
	let docker = Arc::new(Docker::new());
	let semaphore = Arc::new(Semaphore::new(2));

	let addr: SocketAddr = ([127, 0, 0, 1], 8083).into();

	let make_svc = make_service_fn(move |_conn| {
		let docker = docker.clone();
		let semaphore = semaphore.clone();
		async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, docker.clone(), semaphore.clone()))) }
	});

	let server = Server::bind(&addr).serve(make_svc);

	println!("Registry Server listening on {}", addr.to_string().bright_blue());

	if let Err(e) = server.await {
		eprintln!("server error: {}", e);
	}
}