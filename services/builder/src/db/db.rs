use dotenv_codegen::dotenv;
use tokio_postgres::NoTls;

pub async fn connection() -> Result<tokio_postgres::Client, tokio_postgres::Error> {
	let (client, connection) = tokio_postgres::connect(dotenv!("COCKROACH_DB_URL"), NoTls).await?;

	tokio::spawn(async move {
		if let Err(e) = connection.await {
			eprintln!("connection error: {}", e);
		}
	});

	Ok(client)
}