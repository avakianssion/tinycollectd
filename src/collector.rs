//! Module to define behavior of sys info collection.
use serde_json::{Value, json};
use std::process::Command;
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

/// Function to get timestamp
pub fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Function to get hostname
pub fn get_hostname(sys: &System) -> String {
    System::host_name()
        .unwrap_or_else(|| "unknown".to_string())
        .replace('"', "\\\"")
}
/// Function to collect system metrics as single json object.
pub fn get_sysinfo(sys: &System) -> Value {
    json!({
        "timestamp": get_timestamp(),
        "hostname": get_hostname(sys),
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

/// Function to get status of specific systemd services by name
pub fn get_service_status(services: &[String]) -> Vec<Value> {
    let mut results = Vec::new();

    for service in services {
        let status = get_service_active_status(&service);

        let service_status = json!({
            "service_name": service,
            "status": status
        });

        results.push(service_status);
    }

    results
}

/// Get the active status of a service
fn get_service_active_status(service: &str) -> String {
    match Command::new("systemctl")
        .args(&["is-active", service])
        .output()
    {
        Ok(output) => str::from_utf8(&output.stdout)
            .unwrap_or("unknown")
            .trim()
            .to_string(),
        Err(_) => "error".to_string(),
    }
}
