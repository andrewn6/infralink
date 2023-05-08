use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Network {
	pub primary_ipv4: String,
	pub primary_ipv6: String,
}
