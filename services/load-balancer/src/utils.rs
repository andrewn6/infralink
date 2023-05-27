use dotenv_codegen::dotenv;
use indexmap::IndexMap;
use std::collections::HashMap;

// Build a routing table given a latency hashmap
pub fn build_routing_table(latencies: HashMap<String, String>) -> IndexMap<String, String> {
	let current_region = dotenv!("REGION");

	let mut pairs: Vec<(String, u32)> = Vec::new();

	for (region, latency) in latencies {
		// split it by :
		let portions: Vec<&str> = region.split(":").collect();

		if portions.len() == 2 {
			// the first portion will contain the source region, which should be the current region
			let source_region = portions[0];

			// the second portion should be the destination region
			let destination_region = portions[1];

			if source_region == current_region {
				// parse latency as u32 and insert the pair into the vector
				if let Ok(latency) = latency.parse::<u32>() {
					pairs.push((destination_region.to_string(), latency));
				}
			}
		}
	}

	// sort the vector by latency
	pairs.sort_by_key(|pair| pair.1);

	// build the index map
	let mut routing_table: IndexMap<String, String> = IndexMap::new();

	for pair in pairs {
		routing_table.insert(pair.0, pair.1.to_string());
	}

	routing_table
}
