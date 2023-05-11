use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstanceState {
	Starting,
	Running,
	Upgrading,
	Stopping,
	Stopped,
	Terminated,
	Unknown,
}
