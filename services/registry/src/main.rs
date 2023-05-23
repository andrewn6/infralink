use futures_util::StreamExt;
use hyper::body::HttpBody;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Method, Response, Server, StatusCode};
use shiplift::{Docker, PullOptions};

use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;

async fn handle_request(req: Request<Body>, docker: Docker) -> Result<Response<Body>, Infallible> {
	match (req.method(), req.uri().path()) {
		(&Method::POST, "/push") => handle_push(req, docker).await,
		(&Method::GET, "/pull") => handle_pull(req, docker).await,
		_ => Ok(Response::builder()
			.status(StatusCode::NOT_FOUND)
			.body(Body::empty())
			.unwrap()),
	}
}

async fn handle_push(mut req: Request<Body>, docker: Docker) -> Result<Response<Body>, Infallible> {
	let mut body = Vec::new();
	while let Some(chunk) = req.body_mut().data().await {
		let chunk = chunk.unwrap();
		body.extend_from_slice(&chunk);
	}

	// Update this to not be hard coded
	let image_name = "registry/image-name:latest";
	let pull_options = shiplift::PullOptions::builder().image(image_name).build();
	let mut image = docker.images().pull(&pull_options);
	let upload = image.push().await.unwrap();

	let mut stream = upload.into_inner();
	stream.write_all(&body).await.unwrap();
	stream.finish().await.unwrap();

	Ok(Response::new(Body::from("imaged pushed successfully")))
}

async fn handle_pull(_req: Request<Body>, docker: Docker) -> Result<Response<Body>, Infallible> {
	let image_name = "registry/image-name:latest";
	let pull_options = shiplift::PullOptions::builder().image(image_name).build();
	let mut stream = docker.images().pull(&pull_options);

	while let Some(result) = stream.next().await {
		match result {
			Ok(output) => println!("{:?}", output),
			Err(err) => println!("Error: {}", err),
		}
	}
	
	Ok(Response::new(Body::from("image pulled successfully")))
}
#[tokio::main]
async fn main() {
	let docker = Docker::new();

	let addr: SocketAddr = ([127, 0, 0, 1], 8083).into();
}