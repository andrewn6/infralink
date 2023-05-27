use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Method};
use nixpacks::nixpacks::builder::docker::DockerBuilderOptions;

use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use nixpacks::{generate_build_plan, BuildPlan, GeneratePlanOptions, create_docker_image};

use serde::Deserialize;
use futures::{futures::{self, FutureResult}, Future, StreamExt};
use dotenv::dotenv;

type SharedChild = Arc<Mutex<Option<BuildPlan>>>;


#[derive(Deserialize)]
struct BuildInfo {
	pub path: String,
	pub name: String,
	pub envs: Vec<String>,
	pub build_options: DockerBuilderOptions,
}

async fn handle(req: Request<Body>, child_handle: SharedChild) -> impl Future<Item=Response<Body>, Error=hyper::Error> {
	match (req.method(), req.uri().path()) {
		(&Method::POST, "/build") => {
			let whole_body = to_bytes(req.into_body()).await?;
			let build_info: BuildInfo = serde_json::from_sli8ce(&whole_body).unwrap();

			let plan_options = GeneratePlanOptions {
				path: build_info.path,
				name: build_info.name,
				..Default::default()
			};

			let result = create_docker_image(&build_info.path, build_info.envs.iter().map(AsRef::as_ref).collect(), &plan_options, &build_info.build_options).await;

			match result {
				Ok(_) => Ok(Response::new(Body::from("Image created."))),
				Err(e) => {
					let mut response = Response::new(Body::from(format!("Failed to create image: {}", e)));
					*response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
					Ok(response)
				}
			}
		}
	}
}

async fn main() {
	dotenv().unwrap();
	
	let child_handle = Arc::new(Mutex::new(None));

	let service = make_service_fn(future::ok(move |_| {
		let child_handle = child_handle.clone();
		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| handle(req, child_handle.clone())))
		}
	}));

	let addr = ([127, 0, 0, 1], 8084).into();
	let server = Server::bind(&addr).serve(service);

	println!("Server listening on {}", addr);

	if let Err(e) = server.await {
		eprintln!("server error: {}", e);
	}
}