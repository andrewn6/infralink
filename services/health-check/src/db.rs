use dotenv_codegen::dotenv;
use redis::aio::Connection;
use redis::RedisResult;

pub async fn connection() -> RedisResult<Connection> {
	let client = redis::Client::open(dotenv!("MASTER_REDIS_CONNECTION_URL"))?;

	let connection = client.get_async_connection().await?;

	Ok(connection)
}
