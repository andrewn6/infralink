use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, StatusCode, Method, Error};
use hyper::Server;

use nixpacks::nixpacks::builder::docker::DockerBuilderOptions as NixpacksOptions;
use nixpacks::nixpacks::plan::generator::GeneratePlanOptions;
use nixpacks::nixpacks::plan::BuildPlan;
use nixpacks::{create_docker_image, generate_build_plan};

pub mod db;
pub mod logs;

use logs::logs::get_logs;
use db::db::connection;
use serde::{Deserialize};
use dotenv_codegen::dotenv;
use serde_json::json;

use reqwest::Client;
use colored::*;
use std::sync::{Arc};
use chrono::Utc;
use tokio::sync::Mutex;

type SharedChild = Arc<Mutex<Option<BuildPlan>>>;

#[derive(Deserialize)]
struct BuildInfo {
	pub path: String,
	pub name: String,
	pub envs: Vec<String>,
	pub build_options: DockerBuilderOptions,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct DockerBuilderOptions {
    pub name: Option<String>,
    pub out_dir: Option<String>,
    pub print_dockerfile: bool,
    pub tags: Vec<String>,
    pub labels: Vec<String>,
    pub quiet: bool,
    pub cache_key: Option<String>,
    pub no_cache: bool,
    pub inline_cache: bool,
    pub cache_from: Option<String>,
    pub platform: Vec<String>,
    pub current_dir: bool,
    pub no_error_without_start: bool,
    pub incremental_cache_image: Option<String>,
    pub verbose: bool,
}

fn convert_to_nixpacks_options(local_options: &DockerBuilderOptions) -> NixpacksOptions {
	NixpacksOptions {
        name: local_options.name.clone(),
        out_dir: local_options.out_dir.clone(),
        print_dockerfile: local_options.print_dockerfile,
        tags: local_options.tags.clone(),
        labels: local_options.labels.clone(),
        quiet: local_options.quiet,
        cache_key: local_options.cache_key.clone(),
        no_cache: local_options.no_cache,
        inline_cache: local_options.inline_cache,
        cache_from: local_options.cache_from.clone(),
        platform: local_options.platform.clone(),
        current_dir: local_options.current_dir,
        no_error_without_start: local_options.no_error_without_start,
        incremental_cache_image: local_options.incremental_cache_image.clone(),
        verbose: local_options.verbose,
    }
}

async fn handle(req: Request<Body>, child_handle: SharedChild) -> Result<Response<Body>, Error> {
	match (req.method(), req.uri().path()) {
		(&Method::POST, "/build") => {
			let conn = match connection().await {
				Ok(conn) => conn,
				Err(e) => {
					eprintln!("Connection error: {}", e);
					return Ok(Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(Body::from("Internal Server Error"))
						.unwrap());
				}
			};
				
			let whole_body = to_bytes(req.into_body()).await?;
			let build_info: BuildInfo = match serde_json::from_slice(&whole_body) {
				Ok(info) => info,
				Err(_) => {
				let response = Response::builder()
					.status(StatusCode::BAD_REQUEST)
					.body(Body::from("Invalid request body"))
					.unwrap();
				return Ok(response);
				}
			};

			if build_info.path.is_empty() || build_info.name.is_empty() {
				let response = Response::builder()
					.status(StatusCode::BAD_REQUEST)
					.body(Body::from("Missing required fields"))
					.unwrap();
				return Ok(response)
			}
			let plan_options = GeneratePlanOptions::default(); // Generate default options

			let plan = generate_build_plan(
				&build_info.path,
				build_info.envs.iter().map(AsRef::as_ref).collect(),
				&plan_options
			);

			let nixpack_options = convert_to_nixpacks_options(&build_info.build_options);

			let start_time = Utc::now().to_rfc3339();
			let build_if = format!("{}:{}", &build_info.path, &start_time);

			/* Insert build data once build is triggered */
			conn.execute("INSERT into build_data (id, start_time, status) VALUES ($1, $2, $3)",
				&[&build_if, &start_time, &"running"]).await.unwrap();

			let result = create_docker_image(
				&build_info.path,
				build_info.envs.iter().map(AsRef::as_ref).collect(),
				&plan_options,
				&nixpack_options,
			).await;

			let status = match result {
				Ok(_) => {
					let client = Client::new();
					let registry_post_data = json!({
						"registry_url": dotenv!("DOCKER_REGISTRY_URL"),
						"image_name": build_info.name,
						"image_tag": build_info.build_options.tags.get(0).unwrap_or(&"latest".to_string())
					});

					let push_result = client.post("http://localhost:8083/push")
						.json(&registry_post_data)
						.send()
						.await;

					match push_result {
						Ok(_) => "Completed",
						Err(_) => "Failed"
					}
				},
				Err(_) => "Failed"
			};

			let end_time = Utc::now().to_rfc3339();
			conn.execute("UPDATE build_data SET status = $1, end_time = $2 WHERE id = $3",
				&[&status, &end_time, &build_if]).await.unwrap();

			let _ = match result {
				Ok(_) => Ok(Response::new(Body::from("Image created."))),
				Err(e) => Err({
					let mut response = Response::new(Body::from(format!("Failed to create image: {}", e)));
					*response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
					response
				})
			};

			Ok(Response::new(Body::from("Image created.")))
		},
		
		_ => {
			Ok(Response::builder().status(StatusCode::METHOD_NOT_ALLOWED).body(Body::from("Method not allowed")).unwrap())
		}
	}
}

#[tokio::main]
async fn main() {	
	let child_handle = Arc::new(Mutex::new(None));

	let service = make_service_fn(move |_| {
		let child_handle = child_handle.clone();
		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| handle(req, child_handle.clone())))
		}
	});

	let addr = ([127, 0, 0, 1], 8084).into();
	let server = Server::bind(&addr).serve(service);

	println!("Builder Server listening on {}", addr.to_string().bright_blue());

	if let Err(e) = server.await {
		eprintln!("server error: {}", e);
	}
}