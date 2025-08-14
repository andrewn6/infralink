use principal::examples::autoscaling_example::run_autoscaling_example;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_autoscaling_example().await
}