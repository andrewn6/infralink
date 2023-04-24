use sysinfo::{NetworkExt, SystemExt};

use crate::models::metadata::network::NetworkMetadata;

/// Read all information about the network of the system the worker is running on.
///
/// We current measure:
/// - Total outbound bandwidth in bytes since the server started
pub fn network() -> NetworkMetadata {
    let network_metadata = NetworkMetadata::new();

    let mut system = sysinfo::System::new_all();

    system.refresh_networks();

    let mut total_outbound = 0;

    for (_, network) in system.networks() {
        total_outbound += network.total_transmitted();
    }

    let network_metadata = NetworkMetadata {
        total_outbound: Some(total_outbound),
    };

    network_metadata
}
