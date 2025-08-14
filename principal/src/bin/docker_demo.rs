use principal::examples::docker_example::run_docker_example;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_docker_example().await
}