use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CloudProvider {
	Vultr,
	Hetzner,
	Oracle,
	HostHatch,
}
