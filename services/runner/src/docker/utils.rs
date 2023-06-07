use shiplift::tty::TtyChunk;
use shiplift::{ContainerOptions, Docker, PullOptions, RmContainerOptions, LogsOptions};
use shiplift::errors::Error;
use futures::{StreamExt};

pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    pub fn new() -> DockerClient {
        DockerClient {
            client: Docker::new(),
        }
    }
    pub async fn start_container(&self, image: &str) -> Result<String, Error> {
        let options = ContainerOptions::builder(image).build();

        let info = self.client.containers().create(&options).await?;

        self.client.containers().get(&info.id).start().await?;

        Ok(info.id)
    }

    pub async fn stop_container(&self, container_id: &str) -> Result<(), Error> {
        let options = RmContainerOptions::builder().force(true).build();

        self.client.containers().get(container_id).remove(options).await?;
        Ok(())
    }

    pub async fn get_container_status(&self, container_id: &str) -> Result<shiplift::rep::ContainerDetails, Error> {
        let details = self.client.containers().get(container_id).inspect().await?;
        Ok(details)
    }

    pub async fn stream_logs(&self, container_id: &str) -> Result<(), Error> {
        let options = LogsOptions::builder()
            .stdout(true)
            .stderr(true)
            .timestamps(true)
            .follow(true)
            .build();
        
        let mut stream = self.client.containers().get(container_id).logs(&options);
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(TtyChunk::StdOut(bytes)) | Ok(TtyChunk::StdErr(bytes)) => {
                    println!("{}", String::from_utf8_lossy(&bytes))
                },
                Ok(TtyChunk::StdIn(_)) => {}, // ignore stdin
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

}

impl Default for DockerClient {
    fn default() -> Self {
        Self::new()
    }
}