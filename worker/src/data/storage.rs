use std::path::Path;
use sysinfo::{DiskExt, SystemExt};

use crate::models::metadata::storage::{StorageMetadata, Volume};

pub fn storage() -> StorageMetadata {
	let mut storage_metadata = StorageMetadata::new();

	let sys = sysinfo::System::new_with_specifics(
		sysinfo::RefreshKind::new().with_disks().with_disks_list(),
	);

	let mut primary = Volume::new();
	let mut volumes = Vec::new();

	for disk in sys.disks() {
		let mut volume = Volume::new();

		volume.total = Some(disk.total_space());
		volume.free = Some(disk.available_space());
		volume.used = Some(disk.total_space() - disk.available_space());

		// Identify the primary disk based on the mount point
		let mount_point = disk.mount_point();

		if mount_point == Path::new("/") {
			primary = volume.clone();
		} else if disk.is_removable() {
			volumes.push(volume);
		}
	}

	storage_metadata.primary = Some(primary);
	storage_metadata.volumes = Some(volumes);

	storage_metadata
}
