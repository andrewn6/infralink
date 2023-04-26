use tonic::transport::Channel;
use tonic::Request;
//use worker::memory::MemoryMetadata;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let channel = Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;

    //let mut client = MemoryService::MemoryServiceServer::new(channel);

    
    Ok(())
}