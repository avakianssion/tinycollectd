//! Module to define behavior of sys info collection.
use serde_json::{Value, json};
use sysinfo::{Disks, Networks, System};

/// Enum to define categories of data to collect.
#[derive(Copy, Clone, Debug)]
enum Category {
    CpuFreq,
    DiskUsage,
    Interface,
}

/// Struct to define what system metrics to pull based on cli args.
#[derive(Clone, Debug)]
struct Collector {
    /// List of categories to pull.
    categories: Vec<Category>,
}

impl Collector {
    /// Constructor
    fn new(categories: Vec<Category>) -> Self {
        Self { categories }.clone()
    }

    /// Function that collects data.
    pub fn collect(&self) -> Value {
        json!({})
    }

    /// Function to enable collection of all categories.
    fn all_categories() -> Self {
        let categories = Vec::new();
        Self::new(categories)
    }

    fn with_categories(categories: Vec<Category>) -> Self {
        Self::new(categories)
    }
}

/// Function to collect system metrics as single json object.
pub fn get_sysinfo(mut sys: System) -> Value {
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
        "disk_usage": get_disk_usage(),
        "network": get_if_data()
    })
}

/// Function to get metrics from interfaces.
fn get_if_data() -> Vec<Value> {
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
fn get_disk_usage() -> Vec<Value> {
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
