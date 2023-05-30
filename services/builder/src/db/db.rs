use dotenv_codegen::dotenv;
use postgres::{Client, NoTls, Error};

pub async fn connection() -> Result<Client, Error> {
	let client = Client::connect(dotenv!("COCKROACH_DB_URL"), NoTls)?;

	Ok(client)
}