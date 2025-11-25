//! Module to define behavior of sys info collection.
use serde_json::{Value, json};
use sysinfo::{Disks, Networks, System};
use std::process::Command;
use std::fs;
use std::io;

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

// Discover controllers
pub fn list_nvme_controllers() -> io::Result<Vec<String>> {
    let mut names = Vec::new();

    // This dir contains entries like "nvme0", "nvme1", ...
    let entries = match fs::read_dir("/sys/class/nvme") {
        Ok(e) => e,
        Err(e) => {
            eprintln!("No /sys/class/nvme found or not readable: {e}");
            return Ok(names); // return empty list instead of hard error
        }
    };

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        names.push(name);
    }

    Ok(names)
}

/// Function to extract S.M.A.R.T metrics
pub fn print_all_nvme_smart_logs() {
    let controllers = match list_nvme_controllers() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to list NVMe controllers: {e}");
            return;
        }
    };

    if controllers.is_empty() {
        println!("No NVMe controllers detected.");
        return;
    }

    for ctrl in controllers {
        let dev_path = format!("/dev/{ctrl}");
        println!("NVMe SMART for {dev_path}");

        let output = match Command::new("nvme")
            .args(["smart-log", &dev_path, "-o", "json"])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Failed to run nvme on {dev_path}: {e}");
                continue;
            }
        };

        if !output.status.success() {
            eprintln!(
                "nvme smart-log failed for {dev_path}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{stdout}");
    }
}
