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

impl Category {
    fn get_metrics(&self, sys: &System) -> String {
        match self {
            Category::CpuFreq => get_cpu_freq(&sys),
            Category::DiskUsage => get_disk_usage(),
            Category::Interface => get_if_data(),
        }
    }
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

    println!("{}", get_cpu_freq(&sys));
    println!("{}", get_disk_usage());
    println!("{}", get_if_data());

    json!({
        "timestamp": get_timestamp(),
        "hostname": get_hostname(),
        "uptime": get_uptime(),
        "cpu_freq": get_cpu_freq(&sys),
        "disk_usage": get_disk_usage(),
        "network": get_if_data()
    })
}

/// Function to get timestamp of poll.
fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Function to get hostname of system.
fn get_hostname() -> String {
    System::host_name()
        .unwrap_or_else(|| "unknown".to_string())
        .replace('"', "\\\"")
}

/// Function to get current uptime of system.
fn get_uptime() -> u64 {
    System::uptime()
}

/// Function to get metrics from interfaces.
fn get_if_data() -> String {
    let networks = Networks::new_with_refreshed_list();

    json!({"if": networks
        .iter()
        .map(|(name, data)| {
            json!({
                "interface": name.replace('"', "\\\""),
                "rx_bytes": data.total_received(),
                "tx_bytes": data.total_transmitted()
            })
        })
        .collect::<Vec<Value>>()})
    .to_string()
}

/// Function to get cpu frequency information.
fn get_cpu_freq(sys: &System) -> String {
    json!({"cpu_freq": sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0)}).to_string()
}

/// Function to get disk usage information.
fn get_disk_usage() -> String {
    let disks = Disks::new_with_refreshed_list();

    json!({"disk_usage": disks
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
        .collect::<Vec<Value>>()})
    .to_string()
}
