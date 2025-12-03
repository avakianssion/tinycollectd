//! Module to define behavior of sys info collection.
use linux_nvme_sys::{nvme_admin_cmd, nvme_admin_opcode::nvme_admin_get_log_page, nvme_smart_log, nvme_id_ctrl};
use serde::Serialize;
use serde_json::{Value, json};
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::mem::{size_of, zeroed};
use std::os::unix::io::AsRawFd;
use std::process::Command;
use sysinfo::{Disks, Networks, System};

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

/// Helper function to convert a 16-byte little-endian NVMe counter into u64
fn le_16_to_u128(bytes: &[u8; 16]) -> u128 {
    u128::from_le_bytes(*bytes)
}

/// Helper function to convert a 32-bit little-endian NVMe counter into u64
fn le32_to_u64(v: linux_nvme_sys::__le32) -> u64 {
    u32::from(v) as u64
}

/// Function to get raw uptime.
fn uptime_raw(_sys: &System) -> String {
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
pub fn get_hostname(_sys: &System) -> String {
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
        "network": get_if_data(),
        "smart_log": collect_smart_log(),
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
pub fn manual_collect_smart_log() -> Vec<NvmesSmartLog> {
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

/// Function to extract smart log through linux-nvme-sys crate
fn smart_log_from_kernel(nvme_name: String, raw: &nvme_smart_log) -> NvmesSmartLog {
    // temp is 2 bytes, just join as u16?
    let temp = u16::from_le_bytes([raw.temperature[0], raw.temperature[1]]) as u64;

    // first two temp sensors from temp_sensor[]
    let ts1 = u16::from(raw.temp_sensor[0]) as u64;
    let ts2 = u16::from(raw.temp_sensor[1]) as u64;

    // 128-bit counters
    let data_units_read = le_16_to_u128(&raw.data_units_read) as u64;
    let data_units_written = le_16_to_u128(&raw.data_units_written) as u64;
    let host_reads = le_16_to_u128(&raw.host_reads) as u64;
    let host_writes = le_16_to_u128(&raw.host_writes) as u64;
    let ctrl_busy_time = le_16_to_u128(&raw.ctrl_busy_time) as u64;
    let power_cycles = le_16_to_u128(&raw.power_cycles) as u64;
    let power_on_hours = le_16_to_u128(&raw.power_on_hours) as u64;
    let unsafe_shutdowns = le_16_to_u128(&raw.unsafe_shutdowns) as u64;
    let media_errors = le_16_to_u128(&raw.media_errors) as u64;
    let num_err_log_entries = le_16_to_u128(&raw.num_err_log_entries) as u64;

    NvmesSmartLog {
        nvme_name,
        avail_spare: Some(raw.avail_spare as u64),
        controller_busy_time: Some(ctrl_busy_time),
        critical_comp_time: Some(le32_to_u64(raw.critical_comp_time)),
        critical_warning: Some(raw.critical_warning as u64),
        data_units_read: Some(data_units_read),
        data_units_written: Some(data_units_written),
        endurance_grp_critical_warning_summary: None, // not exposed through the create?
        host_read_commands: Some(host_reads),
        host_write_commands: Some(host_writes),
        media_errors: Some(media_errors),
        num_err_log_entries: Some(num_err_log_entries),
        percent_used: Some(raw.percent_used as u64),
        power_cycles: Some(power_cycles),
        power_on_hours: Some(power_on_hours),
        spare_thresh: Some(raw.spare_thresh as u64),
        temperature: Some(temp),
        temperature_sensor_1: Some(ts1),
        temperature_sensor_2: Some(ts2),
        thm_temp1_total_time: Some(le32_to_u64(raw.thm_temp1_total_time)),
        thm_temp1_trans_count: Some(le32_to_u64(raw.thm_temp1_trans_count)),
        thm_temp2_total_time: Some(le32_to_u64(raw.thm_temp2_total_time)),
        thm_temp2_trans_count: Some(le32_to_u64(raw.thm_temp2_trans_count)),
        unsafe_shutdowns: Some(unsafe_shutdowns),
        warning_temp_time: Some(le32_to_u64(raw.warning_temp_time)),
    }
}

/// Get the raw nvme_smart_log from a controller device, e.g. "/dev/nvme0"
pub fn get_nvme_smart_log_raw(dev_path: &str) -> io::Result<nvme_smart_log> {
    let file = OpenOptions::new()
        .read(true)
        .write(true) // admin command may require write
        .open(dev_path)?;

    let fd = file.as_raw_fd();

    let mut log: nvme_smart_log = unsafe { zeroed() };

    let log_ptr = &mut log as *mut nvme_smart_log as u64;
    let log_len = size_of::<nvme_smart_log>() as u32;

    // NVMe spec:
    //   CDW10 bits:
    //     [7:0]  = LID (log id)     -> 0x02 for SMART / health
    //     [31:16] = NUMD (#dwords - 1)
    //
    // smart log is 512 bytes -> 512 / 4 = 128 dwords -> NUMD = 127
    let log_id: u8 = 0x02;
    let numd: u32 = (log_len / 4 - 1).into();
    let cdw10: u32 = (log_id as u32) | (numd << 16);

    let mut cmd: nvme_admin_cmd = unsafe { zeroed() };
    cmd.opcode = nvme_admin_get_log_page as u8;
    cmd.nsid = 0xFFFF_FFFF; // SMART is controller-level; nsid 0xFFFF_FFFF
    cmd.addr = log_ptr;
    cmd.data_len = log_len;
    cmd.cdw10 = cdw10;
    cmd.cdw11 = 0;
    cmd.timeout_ms = 1000;

    let ret = unsafe { linux_nvme_sys::nvme_ioctl_admin_cmd(fd, &mut cmd) };

    match ret {
        Ok(status) if status == 0 => Ok(log),
        Ok(status) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("NVMe admin command failed, status={:#x}", status),
        )),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
    }
}

/// Function to collect extracted smart log data
pub fn collect_smart_log() -> Vec<NvmesSmartLog> {
    let mut results = Vec::new();
    let ctrls = list_nvme_controllers();

    for ctrl in ctrls {
        let dev_path = format!("/dev/{}", ctrl);

        match get_nvme_smart_log_raw(&dev_path) {
            Ok(raw) => {
                let mapped = smart_log_from_kernel(ctrl.clone(), &raw);
                results.push(mapped);
            }
            Err(e) => {
                eprintln!("Failed to fetch SMART log for {}: {}", dev_path, e);
            }
        }
    }

    results
}