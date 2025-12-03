// src/collector/net.rs
//! Network interface metrics.

use serde_json::{Value, json};
use sysinfo::Networks;

/// Function to extract interface data.
pub fn get_if_data() -> Vec<Value> {
    let networks = Networks::new_with_refreshed_list();

    networks
        .iter()
        .map(|(name, data)| {
            json!({
                "interface": name.replace('"', "\\\""),
                "rx_bytes": data.total_received(),
                "tx_bytes": data.total_transmitted()
            })
        })
        .collect()
}
