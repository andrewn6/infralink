use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bandwidth {
	incoming_bytes: u64,
	outgoing_bytes: u64,
}
