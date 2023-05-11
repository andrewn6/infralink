use dotenv_codegen::dotenv;
use redis::cluster::ClusterClient;
use redis::cluster_async::ClusterConnection;
use redis::RedisResult;

pub async fn connection() -> RedisResult<ClusterConnection> {
	let nodes = vec![dotenv!("NEW_YORK_REDIS_CONNECTION_URL")];

	let client = ClusterClient::new(nodes).unwrap();

	let connection = client.get_async_connection().await?;

	Ok(connection)
}
