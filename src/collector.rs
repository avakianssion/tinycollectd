//! Module to define behavior of sys info collection.
use serde::Serialize;
use serde_json::{Value, json};
use std::fs;
use std::io;
use std::process::Command;
use sysinfo::{Disks, Networks, System};

// TODO - This struct is incomplete - There are other important things in smart info logs that I think we should keep
// Serialize smart log metrics
#[derive(Debug, Serialize)]
pub struct NvmesSmartLog {
    pub nvme_name: String, // tag the nvme_name
    pub avail_spare: Option<u64>,
    pub controller_busy_time: Option<u64>,
    pub critical_comp_time: Option<u64>,
    pub critical_warning: Option<u64>,
    pub data_units_read: Option<u64>,
    pub data_units_written: Option<u64>,
    pub endurance_grp_critical_warning_summary: Option<u64>,
    pub host_read_commands: Option<u64>,
    pub host_write_commands: Option<u64>,
    pub media_errors: Option<u64>,
    pub num_err_log_entries: Option<u64>,
    pub percent_used: Option<u64>,
    pub power_cycles: Option<u64>,
    pub power_on_hours: Option<u64>,
    pub spare_thresh: Option<u64>,
    pub temperature: Option<u64>,
    pub temperature_sensor_1: Option<u64>,
    pub temperature_sensor_2: Option<u64>,
    pub thm_temp1_total_time: Option<u64>,
    pub thm_temp1_trans_count: Option<u64>,
    pub thm_temp2_total_time: Option<u64>,
    pub thm_temp2_trans_count: Option<u64>,
    pub unsafe_shutdowns: Option<u64>,
    pub warning_temp_time: Option<u64>,
}

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

/// Function to discover controllers exposed on the server
pub fn list_nvme_controllers() -> Vec<String> {
    let mut names = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/nvme") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            names.push(name);
        }
    }

    names
}

/// Function to extract S.M.A.R.T metrics
/// Eventually, I want the other events to be their own structs like this one
pub fn collect_smart_log() -> Vec<NvmesSmartLog> {
    let mut results = Vec::new();
    let ctrls = list_nvme_controllers();

    for ctrl in ctrls {
        let path = format!("/dev/{ctrl}");

        let output = match Command::new("nvme")
            .args(["smart-log", &path, "-o", "json"])
            .output()
        {
            Ok(o) if o.status.success() => o,
            Ok(o) => {
                eprintln!(
                    "nvme smart-log failed for {}: {}",
                    path,
                    String::from_utf8_lossy(&o.stderr)
                );
                continue;
            }
            Err(e) => {
                eprintln!("Failed to run nvme on {path}: {e}");
                continue;
            }
        };

        let raw_json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Failed to parse JSON for {}: {e}", path);
                continue;
            }
        };

        // Extract common fields
        let entry = NvmesSmartLog {
            nvme_name: ctrl,

            avail_spare: raw_json.get("avail_spare").and_then(|v| v.as_u64()),

            controller_busy_time: raw_json
                .get("controller_busy_time")
                .and_then(|v| v.as_u64()),

            critical_comp_time: raw_json.get("critical_comp_time").and_then(|v| v.as_u64()),

            critical_warning: raw_json.get("critical_warning").and_then(|v| v.as_u64()),

            data_units_read: raw_json.get("data_units_read").and_then(|v| v.as_u64()),

            data_units_written: raw_json.get("data_units_written").and_then(|v| v.as_u64()),

            endurance_grp_critical_warning_summary: raw_json
                .get("endurance_grp_critical_warning_summary")
                .and_then(|v| v.as_u64()),

            host_read_commands: raw_json.get("host_read_commands").and_then(|v| v.as_u64()),

            host_write_commands: raw_json.get("host_write_commands").and_then(|v| v.as_u64()),

            media_errors: raw_json.get("media_errors").and_then(|v| v.as_u64()),

            num_err_log_entries: raw_json.get("num_err_log_entries").and_then(|v| v.as_u64()),

            percent_used: raw_json.get("percent_used").and_then(|v| v.as_u64()),

            power_cycles: raw_json.get("power_cycles").and_then(|v| v.as_u64()),

            power_on_hours: raw_json.get("power_on_hours").and_then(|v| v.as_u64()),

            spare_thresh: raw_json.get("spare_thresh").and_then(|v| v.as_u64()),

            temperature: raw_json.get("temperature").and_then(|v| v.as_u64()),

            temperature_sensor_1: raw_json
                .get("temperature_sensor_1")
                .and_then(|v| v.as_u64()),

            temperature_sensor_2: raw_json
                .get("temperature_sensor_2")
                .and_then(|v| v.as_u64()),

            thm_temp1_total_time: raw_json
                .get("thm_temp1_total_time")
                .and_then(|v| v.as_u64()),

            thm_temp1_trans_count: raw_json
                .get("thm_temp1_trans_count")
                .and_then(|v| v.as_u64()),

            thm_temp2_total_time: raw_json
                .get("thm_temp2_total_time")
                .and_then(|v| v.as_u64()),

            thm_temp2_trans_count: raw_json
                .get("thm_temp2_trans_count")
                .and_then(|v| v.as_u64()),

            unsafe_shutdowns: raw_json.get("unsafe_shutdowns").and_then(|v| v.as_u64()),

            warning_temp_time: raw_json.get("warning_temp_time").and_then(|v| v.as_u64()),
        };

        results.push(entry);
    }

    results
}
