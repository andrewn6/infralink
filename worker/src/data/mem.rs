use sysinfo::SystemExt;

use crate::models::metadata::memory::{Memory, MemoryMetadata};

/// Read all information about the memory of the system the worker is running on.
///
/// We current measure:
/// - Total memory available to the system
/// - Total memory used by the system
/// - Total memory free in the system
pub fn memory() -> MemoryMetadata {
    let mut memory_metadata = MemoryMetadata::new();

    // Get information about the primary memory installed onto the system
    let mut primary = Memory::new();

    let sys = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_memory());

    primary.total = Some(sys.total_memory());
    primary.used = Some(sys.used_memory());
    primary.free = Some(sys.available_memory());

    // Get information about the swap memory installed onto the system
    let mut swap = Memory::new();

    swap.total = Some(sys.total_swap());
    swap.used = Some(sys.used_swap());
    swap.free = Some(sys.free_swap());

    // Populate the memory metadata
    memory_metadata.primary = Some(primary);
    memory_metadata.swaps = Some(swap);

    memory_metadata
}
