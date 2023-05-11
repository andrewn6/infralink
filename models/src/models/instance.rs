use serde::{Deserialize, Serialize};

use super::volume::Volume;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
	pub vcpu: u64,
	pub memory: u64,
	pub boot_volume: Volume,
}
