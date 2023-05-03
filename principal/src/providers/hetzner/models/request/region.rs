use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Region {
	Falkenstein,
	Nuremberg,
	Helsinki,
	Ashburn,
	Hillsboro,
	Unknown,
}

impl ToString for Region {
	fn to_string(&self) -> String {
		match self {
			Region::Falkenstein => "Falkestein".to_string(),
			Region::Nuremberg => "Nuremberg".to_string(),
			Region::Helsinki => "Helsinki".to_string(),
			Region::Ashburn => "Ashburn".to_string(),
			Region::Hillsboro => "Hillsboro".to_string(),
			Region::Unknown => "Unknown".to_string(),
		}
	}
}

impl Region {
	fn code(self) -> String {
		match self {
			Region::Falkenstein => "fsn1".to_string(),
			Region::Nuremberg => "nbg1".to_string(),
			Region::Helsinki => "hel1".to_string(),
			Region::Ashburn => "ash".to_string(),
			Region::Hillsboro => "hil".to_string(),
			Region::Unknown => "Unknown".to_string(),
		}
	}
}
