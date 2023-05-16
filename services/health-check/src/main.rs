use tonic::{Request, Response, Status};
use tonic::transport::Server;

use dotenv::dotenv;

pub mod db;
pub mod health_check;

mod worker_info {
	include!("worker_info.rs");
}

use worker_info::health_check_service_server::HealthCheckService;
use worker_info::{WorkerInfo, ScheduleHealthCheckResponse};

use worker_info::health_check_service_server;

#[derive(Default, Clone)]
pub struct HealthCheckServer;

impl tonic::server::NamedService for HealthCheckServer {
	const NAME: &'static str = "health_check.HealthCheck";
}

#[tonic::async_trait]
impl HealthCheckService for HealthCheckServer {
    async fn schedule_health_checks(
        &self,
        request: Request<WorkerInfo>,
    ) -> Result<Response<ScheduleHealthCheckResponse>, Status> {
        println!("Got a request: {:?}", request);
        
        let reply = worker_info::ScheduleHealthCheckResponse {
            message: format!("Health check scheduled for worker with ID: {}", request.into_inner().id),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
pub async fn main() {
	dotenv().unwrap();

	let mut connection = db::connection().await.unwrap();

	/* Used for testing
	let tasks_map: Arc<Mutex<HashMap<String, HealthCheckTask>>> =
		Arc::new(Mutex::new(HashMap::new()));

	schedule_health_checks(
		&mut connection,
		1,
		WorkerInfo {
			id: 123,
			network: Network {
				primary_ipv4: String::from("139.180.143.66"),
				primary_ipv6: String::new(),
			},
		},
		vec![HealthCheck {
			path: "/health".to_string(),
			port: 80,
			method: Some(HttpMethod::GET),
			tls_skip_verification: None,
			grace_period: 0,
			interval: 4000,
			timeout: 10000,
			max_failures: 2,
			r#type: HealthCheckType::HTTP,
			headers: Some(vec![Header {
				key: "Host".to_string(),
				value: "edge.dimension.dev".to_string(),
			}]),
		}],
		tasks_map.clone(),
	)
	.await;
	*/

	let addr = "0.0.0.0:50052".parse().unwrap();
	let health_check_server = health_check_service_server::HealthCheckServiceServer::new(HealthCheckServer);

	println!("Health Check Server listening on {}", addr);
	Server::builder()
		.add_service(health_check_server)
		.serve(addr)
		.await
		.unwrap();
}
