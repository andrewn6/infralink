use principal::examples::ingress_example::run_ingress_example;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_ingress_example().await
}