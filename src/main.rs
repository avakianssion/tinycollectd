// src/main.rs
use serde_json::{json, Value};
use std::time::Duration;
use sysinfo::{Disks, Networks, System};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get target from environment or use default
    let target = std::env::var("METRICS_TARGET").unwrap_or_else(|_| "127.0.0.1:1555".to_string());

    println!("Sending metrics to UDP {}", target);

    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    loop {
        let bytes = serde_json::to_vec(&collect_sysinfo(System::new_all())).unwrap();

        // Send UDP packet
        if let Err(e) = socket.send_to(&bytes, &target).await {
            eprintln!("Failed to send UDP packet: {}", e);
        } else {
            println!("Sent metrics to {} ({} bytes)", target, &bytes.len());
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

/// Function to collect system metrics as single json object.
fn collect_sysinfo(mut sys: System) -> Value {
    sys.refresh_all();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let hostname = System::host_name()
        .unwrap_or_else(|| "unknown".to_string())
        .replace('"', "\\\"");
    let uptime = System::uptime();
    let cpu_freq = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);

    json!({
        "timestamp": timestamp,
        "hostname": hostname,
        "uptime": uptime,
        "cpu_freq_mhz": cpu_freq,
        "disk_usage": collect_usage(),
        "network": collect_net()
    })
}

/// Function to get network data from system.
fn collect_net() -> Vec<Value> {
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
// TODO: do collector module
/// Function to get disk usage data from system.
fn collect_usage() -> Vec<Value> {
    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let used_percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            json!({
                "mount": disk.mount_point().to_string_lossy().replace('"', "\\\""),
                "total_gb": total / 1_000_000_000,
                "used_gb": used / 1_000_000_000,
                "used_percent": used_percent
            })
        })
        .collect()
}
