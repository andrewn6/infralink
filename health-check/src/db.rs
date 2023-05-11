use dotenv_codegen::dotenv;
use redis::RedisResult;

pub fn connection() -> RedisResult<redis::Connection> {
	let client = redis::Client::open(dotenv!("REDIS_CONNECTION"))?;

	let con = client.get_connection()?;

	Ok(con)
}
