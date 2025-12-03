// src/collector/sys.rs
//! System-level info: timestamp, hostname, uptime, cpu freq, top-level sysinfo.

use serde_json::{Value, json};
use sysinfo::System;

/// Function to generate a timestamp in epoch time.
pub fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Function to extract hostname of the system.
pub fn get_hostname() -> String {
    System::host_name()
        .unwrap_or_else(|| "unknown".to_string())
        .replace('"', "\\\"")
}

/// Function to extract raw uptime string.
fn uptime_raw() -> String {
    System::uptime().to_string()
}

/// Function t get extract raw cpu freq string.
fn cpu_freq_raw(sys: &System) -> String {
    let cpu_freq = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);
    cpu_freq.to_string()
}

/// Function to extract top level system information.
pub fn get_sysinfo(sys: &System) -> Value {
    json!({
        "timestamp": get_timestamp(),
        "hostname": get_hostname(),
        "uptime": uptime_raw(),
        "cpu_freq_mhz": cpu_freq_raw(sys),
        "disk_usage": crate::collector::disk::get_disk_usage(),
        "network": crate::collector::net::get_if_data(),
        "smart_log": crate::collector::nvme::collect_smart_log(),
    })
}

/// Wrapper function for uptime.
pub fn uptime_json() -> Value {
    json!({ "uptime": uptime_raw() })
}

/// Wrapper function for cpu  freq.
pub fn cpu_freq_json(sys: &System) -> Value {
    json!({ "cpu_freq_mhz": cpu_freq_raw(sys) })
}
