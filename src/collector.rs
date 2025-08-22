//! Module to define behavior of sys info collection.
use serde_json::{Value, json};
use sysinfo::{Disks, Networks, System};

/// Function to get raw uptime.
fn uptime_raw(sys: &System) -> String {
    System::uptime().to_string()
}
/// Function to get raw cpufreq.
fn cpu_freq_raw(sys: &System) -> String {
    let cpu_freq = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);
    cpu_freq.to_string()
}
/// Function to collect system metrics as single json object.
pub fn get_sysinfo(sys: &System) -> Value {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let hostname = System::host_name()
        .unwrap_or_else(|| "unknown".to_string())
        .replace('"', "\\\"");

    json!({
        "timestamp": timestamp,
        "hostname": hostname,
        "uptime": uptime_raw(sys),
        "cpu_freq_mhz": cpu_freq_raw(sys),
        "disk_usage": get_disk_usage(),
        "network": get_if_data()
    })
}
/// Function to get JSON formatted uptime.
pub fn uptime_json(sys: &System) -> Value {
    json!({"uptime": uptime_raw(sys)})
}
/// Function to get JSON formatted cpufreq.
pub fn cpu_freq_json(sys: &System) -> Value {
    json!({"cpu_freq_mhz": cpu_freq_raw(sys)})
}
/// Function to get metrics from interfaces.
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
/// Function to get disk usage information.
pub fn get_disk_usage() -> Vec<Value> {
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
