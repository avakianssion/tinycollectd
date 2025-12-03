// src/collector/mod.rs

pub mod disk;
pub mod net;
pub mod nvme;
pub mod services;
pub mod sys;

pub use sys::{cpu_freq_json, get_hostname, get_sysinfo, get_timestamp, uptime_json};

pub use disk::get_disk_usage;
pub use net::get_if_data;

pub use services::get_service_status;

pub use nvme::{NvmesSmartLog, collect_smart_log, list_nvme_controllers};
