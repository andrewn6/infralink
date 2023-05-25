use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::providers::vultr::models::request::instance::InstanceType;

#[derive(Clone, Debug, PartialEq)]
pub enum Plan {
	HighFrequency(Compute),
	HighPerformance(Compute),
	GeneralPurpose(Compute),
	CPUOptimized(Compute),
	Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Compute {
	pub vcpu: u16,
	pub ram: u32,
	pub disk: Option<u32>,
}

impl Plan {
	// Move the plan list to a separate function to make the code DRY
	fn general_purpose_plans() -> Vec<Self> {
		vec![
			Plan::GeneralPurpose(Compute {
				vcpu: 1,
				ram: 4096,
				disk: Some(30),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 2,
				ram: 8192,
				disk: Some(50),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 4,
				ram: 16384,
				disk: Some(80),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 8,
				ram: 32768,
				disk: Some(160),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 16,
				ram: 65536,
				disk: Some(320),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 24,
				ram: 98304,
				disk: Some(480),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 32,
				ram: 131072,
				disk: Some(640),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 40,
				ram: 163840,
				disk: Some(768),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 64,
				ram: 196608,
				disk: Some(960),
			}),
			Plan::GeneralPurpose(Compute {
				vcpu: 96,
				ram: 261120,
				disk: Some(1280),
			}),
		]
	}

	fn cpu_optimized_plans() -> Vec<Self> {
		vec![
			Plan::CPUOptimized(Compute {
				vcpu: 1,
				ram: 2048,
				disk: Some(25),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 2,
				ram: 4096,
				disk: Some(50),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 2,
				ram: 4096,
				disk: Some(75),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 4,
				ram: 8192,
				disk: Some(75),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 4,
				ram: 8192,
				disk: Some(150),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 8,
				ram: 16384,
				disk: Some(150),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 8,
				ram: 16384,
				disk: Some(300),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 16,
				ram: 32768,
				disk: Some(300),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 16,
				ram: 32768,
				disk: Some(500),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 32,
				ram: 65536,
				disk: Some(500),
			}),
			Plan::CPUOptimized(Compute {
				vcpu: 32,
				ram: 65536,
				disk: Some(1000),
			}),
		]
	}

	pub fn list(instance_type: InstanceType) -> Vec<Self> {
		match instance_type {
			InstanceType::GeneralPurpose => Self::general_purpose_plans(),
			InstanceType::CPUOptimized => Self::cpu_optimized_plans(),
			_ => vec![],
		}
	}

	// Find a plan that matches the given compute
	fn find(instance_type: InstanceType, comparison_compute: Compute) -> Option<Self> {
		Self::list(instance_type)
			.into_iter()
			.find(|plan| match plan {
				Plan::GeneralPurpose(compute) | Plan::CPUOptimized(compute) => {
					compute.vcpu == comparison_compute.vcpu && compute.ram == comparison_compute.ram
				}
				_ => false,
			})
	}

	pub fn code(&self) -> String {
		match self {
			Plan::HighFrequency(compute) => {
				format!(
					"{}-{}c-{}gb",
					InstanceType::HighFrequency.to_string(),
					compute.vcpu,
					compute.ram
				)
			}
			Plan::HighPerformance(compute) => {
				format!(
					"{}-{}c-{}gb-intel",
					InstanceType::HighPerformance.to_string(),
					compute.vcpu,
					compute.ram
				)
			}
			Plan::GeneralPurpose(compute) => {
				// Use the find_plan function to get the matching plan
				let plan = Self::find(InstanceType::GeneralPurpose, compute.clone());

				// Use unwrap_or_else to provide a default value in case the plan is not found
				let disk_size = plan.map_or(0, |p| match p {
					Plan::GeneralPurpose(c) => c.disk.unwrap(),
					_ => 0,
				});

				format!(
					"{}-{}c-{}gb-{}s-amd",
					InstanceType::GeneralPurpose.to_string(),
					compute.vcpu,
					compute.ram,
					disk_size
				)
			}
			Plan::CPUOptimized(compute) => {
				let plan = Self::find(InstanceType::CPUOptimized, compute.clone());

				let disk_size = plan.map_or(0, |p| match p {
					Plan::CPUOptimized(c) => c.disk.unwrap(),
					_ => 0,
				});

				format!(
					"{}-{}c-{}gb-{}s-amd",
					InstanceType::CPUOptimized.to_string(),
					compute.vcpu,
					compute.ram,
					disk_size
				)
			}
			_ => String::new(),
		}
	}

	pub fn from_code(code: &str) -> Result<Self, &'static str> {
		let parts: Vec<&str> = code.split('-').collect();

		if parts.len() != 4 {
			return Err("Invalid code format");
		}

		let instance_type =
			InstanceType::from_str(parts[0]).map_err(|_| "Invalid instance type")?;

		let vcpu = parts[1][..parts[1].len() - 1]
			.parse::<u16>()
			.map_err(|_| "Invalid vCPU count")?;
		let ram = parts[2][..parts[2].len() - 2]
			.parse::<u32>()
			.map_err(|_| "Invalid RAM size")?;
		let disk = parts[3][..parts[3].len() - 1]
			.parse::<u32>()
			.map_err(|_| "Invalid disk size")?;

		let compute = Compute {
			vcpu,
			ram,
			disk: Some(disk),
		};

		let plan = match instance_type {
			InstanceType::HighFrequency => Self::HighFrequency(compute),
			InstanceType::HighPerformance => Self::HighPerformance(compute),
			InstanceType::GeneralPurpose => Self::GeneralPurpose(compute),
			InstanceType::CPUOptimized => Self::CPUOptimized(compute),
		};

		Ok(plan)
	}
}

// Implement Serialize trait for Plan
impl Serialize for Plan {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let code = self.code();
		serializer.serialize_str(&code)
	}
}

// Implement Deserialize trait for Plan
impl<'de> Deserialize<'de> for Plan {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let code = String::deserialize(deserializer)?;

		// Add your logic here to convert the code back into a Plan
		// For example, you might have a function like `Plan::from_code(code: &str) -> Result<Plan, YourErrorType>`
		Plan::from_code(&code).map_err(serde::de::Error::custom)
	}
}
