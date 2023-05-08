// /*
//    gRPC service that interacts with Podman, it creates containers & starts them.
//    uses podman_api crate to interact with Podman.
// */
// use std::collections::HashMap;
// use std::env;
// use std::path::PathBuf;

// use podman_api::models::ContainerCreateCreatedBody;
// use tonic::Request;
// use tonic::Response;
// use tonic::Status;

// // Podman rust bindings
// use podman_api::models::PortMapping;
// use podman_api::opts::ContainerCreateOpts;
// use podman_api::Podman;

// mod podman {
// 	include!("../podman.rs");
// }
// // Imports from generated protobuf to Rust.
// use podman::{CreatePodResponse, Pod, StartContainerRequest, StartContainerResponse};

// #[derive(Default)]
// struct ContainerCreateInfo {
// 	image: String,
// 	name: String,
// 	cmd: Vec<String>,
// 	env: HashMap<String, String>,
// 	ports: Vec<PortMapping>,
// }

// #[tonic::async_trait]
// pub trait ContainerCreateService {
// 	async fn create_pod(
// 		&self,
// 		request: Request<CreatePodResponse>,
// 	) -> Result<Response<StartContainerResponse>, Status>;

// 	async fn start_container(
// 		&self,
// 		request: Request<StartContainerRequest>,
// 	) -> Result<Response<StartContainerResponse>, Status>;
// }

// fn get_socket_path() -> Result<PathBuf, std::io::Error> {
// 	let path = env::var("PODMAN_SOCKET_PATH").unwrap_or("/var/run/podman/podman.sock".to_string());
// 	Ok(PathBuf::from(path))
// }

// impl dyn ContainerCreateService {
// 	async fn create_pod(
// 		&self,
// 		request: Request<Pod>,
// 	) -> Result<Response<CreatePodResponse>, Status> {
// 		let socket_path = get_socket_path()?;
// 		let mut client = Podman::unix(&socket_path);

// 		let create_pod_request = request.into_inner();

// 		let container = create_pod_request
// 			.containers
// 			.first()
// 			.ok_or_else(|| Status::invalid_argument("One container must be specified"))?;

// 		let image = container.image.clone();
// 		let name = container.name.clone();
// 		let commands = container.commands.clone();
// 		let ports = container.ports.clone();
// 		let env = container.env.clone();

// 		let create_container_info = ContainerCreateInfo {
// 			image,
// 			name,
// 			cmd: commands,
// 			env: HashMap::new(),
// 			ports: ports
// 				.iter()
// 				.map(|port_str| {
// 					let port = port_str.parse().map_err(|_| {
// 						Status::invalid_argument(format!("Invalid port: {}", port_str))
// 					})?;
// 					Ok(PortMapping {
// 						host_ip: None,
// 						host_port: Some(port),
// 						container_port: Some(port),
// 						protocol: Some("tcp".to_owned()),
// 						range: Some(0),
// 					})
// 				})
// 				.collect::<Result<Vec<_>, podman_api::errors::Error>>()?,
// 			..Default::default()
// 		};

// 		let create_container_opts = ContainerCreateOpts::builder()
//             .image("my_image_name_or_id")
//             .build();

// 		let response = client.containers().create(&create_container_opts);
// 		let create_container_response: ContainerCreateCreatedBody = match response.await {
// 			Ok(x) => x,
// 			Err(err) => {
// 				return Err(Status::internal(format!(
// 					"Failed to create container: {}",
// 					err
// 				)))
// 			}
// 		};
// 		let create_pod_response = CreatePodResponse {
// 			message: vec![create_container_response]
// 				.into_iter()
// 				.map(|container| container.id)
// 				.collect::<Vec<_>>()
// 				.join(",")
// 				.into()
// 		};

// 		Ok(Response::new(create_pod_response))
// 	}

// 	async fn start_container(
//         &self,
//         request: Request<StartContainerRequest>,
//     ) -> Result<Response<StartContainerResponse>, Status> {
//         let socket_path = get_socket_path()?;
//         let client = Podman::unix(&socket_path);

// 		let container_id = request.into_inner().container_id;

// 		let container = client.containers().get(container_id.to_owned());
// 		let response = container.start(None);

//         // Convert the response to the expected type

// 		let start_container_response = match response.await {
// 			Ok(response) => {
// 				StartContainerResponse {
// 				  message: container_id
// 				}
// 			  }
// 			Err(err) => {
// 				println!("Could not start container: {}", err);
// 				return Err(Status::internal("Failed to start container"));
// 			}
// 		};

//         let response = Response::new(start_container_response);

//         Ok(response)
//     }
// }
