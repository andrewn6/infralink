use principal::examples::storage_example::run_storage_example;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_storage_example().await
}