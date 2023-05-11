use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Volume {
	pub id: u64,
	pub used: u64,
	pub total: u64,
	pub r#type: VolumeType,
	pub tier: VolumeTier,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VolumeTier {
	HighPerformance,
	UltraHighPerformance,
	ExtremePerformance,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VolumeType {
	NVME,
	SATA,
	HDD,
}
