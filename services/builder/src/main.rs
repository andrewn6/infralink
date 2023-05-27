pub mod db;
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Method};
use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use serde::Deserialize;

use futures::{futures::{self, FutureResult}, Future, StreamExt};
use dotenv::dotenv;

type SharedChild = Arc<Mutex<Option<Child>>>;


#[derive(Deserialize)]
struct BuildInfo {
	pub path: String,
	pub name: String,
}

fn handle(req: Request<Body>, child_handle: SharedChild) -> impl Future<Item=Response<Body>, Error=hyper::Error> {
	match (req.method(), req.uri().path()) {
		(&Method::POST, "/build") => {
			let whole_body = to_bytes(req.into_body()).await?;
			let build_info: BuildInfo = serde_json::from_slice(&whole_body).unwrap();

			let mut child = Command::new("nixpacks")
				.arg("build")
				.arg(&build_info.path)
				.arg("--name")
				.arg(&build_info.name)
				.spawn()
				.expect("Failed to build");

		    *child_handle.lock().unwrap( )= Some(child);

			Ok(Response::new(Body::from("Build process started.")))
		}
	}
}

fn main() {
	dotenv().unwrap();
	
	let child_handle = Arc::new(Mutex::new(None));

	let service = make_service_fn(future::ok(move |_| {
		let child_handle = child_handle.clone();
		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| handle(req, child_handle.clone())))
		}
	}));

	let addr = ([127, 0, 0, 1], 8083).into();
	let server = Server::bind(&addr).serve(service);

	println!("Server listening on {}", addr);

	if let Err(e) = server.await {
		eprintln!("server error: {}", e);
	}
}