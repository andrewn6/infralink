#[derive(Debug, Clone)]
pub struct StorageMetadata {
	pub primary: Option<Volume>,
	pub volumes: Option<Vec<Volume>>,
}

#[derive(Debug, Clone)]
pub struct Volume {
	pub total: Option<u64>,
	pub used: Option<u64>,
	pub free: Option<u64>,
}

impl Volume {
	pub fn new() -> Self {
		Self {
			total: None,
			used: None,
			free: None,
		}
	}
}

impl StorageMetadata {
	pub fn new() -> Self {
		Self {
			primary: None,
			volumes: None,
		}
	}
}
