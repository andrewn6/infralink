use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    commits: Vec<Commit>, 
}

#[derive(Debug, Deserialize)]
struct Commit {
    id: String,
    added: Vec<String>,
    removed: Vec<String>,
    modified: Vec<String>,
}

async fn handle_webhook(payload: WebhookPayload) {
    println!("{:?}", payload);
}