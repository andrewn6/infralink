use sysinfo::{CpuExt, SystemExt};

use crate::models::metadata::compute::{ComputeMetadata, Cpu};

/// Read all information about the usage of the compute of the system the worker is running on.
///
/// We current measure:
/// - Number of CPU cores
/// - CPU frequency (in MHz)
/// - CPU load (percentage)
pub fn compute_usage() -> ComputeMetadata {
	let mut compute_metadata = ComputeMetadata::new();

	let mut sys = sysinfo::System::new_with_specifics(
		sysinfo::RefreshKind::new().with_cpu(sysinfo::CpuRefreshKind::new().with_cpu_usage()),
	);

	sys.refresh_cpu();

	// We need to sleep for a bit to get the CPU usage
	std::thread::sleep(std::time::Duration::from_millis(250));

	sys.refresh_cpu();

	let cpus = sys.cpus();

	compute_metadata.num_cores = Some(cpus.len() as u64);

	let mut cpus_metadata = vec![];

	for cpu in cpus {
		cpus_metadata.push(Cpu {
			frequency: Some(cpu.frequency()),
			load: Some(cpu.cpu_usage()),
		});
	}

	compute_metadata.cpus = Some(cpus_metadata);

	compute_metadata
}
