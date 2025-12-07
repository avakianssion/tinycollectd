// src/collector/nvme.rs
//! NVMe SMART collection via linux_nvme_sys.

use linux_nvme_sys::{nvme_admin_cmd, nvme_admin_opcode::nvme_admin_get_log_page, nvme_smart_log};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io;
use std::mem::{size_of, zeroed};
use std::os::unix::io::AsRawFd;

#[derive(Debug, Serialize)]
pub struct NvmesSmartLog {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Critical Warning bitmask (Byte 00):
    /// Bit 0: Available spare below threshold
    /// Bit 1: Temperature threshold condition
    /// Bit 2: NVM subsystem degraded reliability
    /// Bit 3: All media read-only
    /// Bit 4: Volatile memory backup failed
    /// Bit 5: Persistent memory region read-only
    /// Bit 6: Indeterminate personality state
    /// Bit 7: Reserved
    /// A value of 0 means no critical warnings
    pub critical_warning: Option<u64>,

    /// Composite Temperature (Bytes 02:01):
    /// Current temperature in Kelvins representing the composite temperature
    /// of the controller and associated namespaces
    pub temperature: Option<u64>,

    /// Available Spare (Byte 03):
    /// Normalized percentage (0-100%) of remaining spare capacity available
    pub avail_spare: Option<u64>,

    /// Available Spare Threshold (Byte 04):
    /// When Available Spare falls below this threshold, an asynchronous event may occur
    /// Normalized percentage (0-100%)
    pub spare_thresh: Option<u64>,

    /// Percentage Used (Byte 05):
    /// Vendor specific estimate of the percentage of NVM subsystem life used
    /// Value of 100 indicates estimated endurance has been consumed
    /// May exceed 100. Updated once per power-on hour
    pub percent_used: Option<u64>,

    /// Endurance Group Critical Warning Summary (Byte 06):
    /// Bit 0: Endurance Group available spare capacity below threshold
    /// Bit 1: Reserved
    /// Bit 2: Endurance Group degraded reliability
    /// Bit 3: Endurance Group read-only
    /// Bits 4-7: Reserved
    pub endurance_grp_critical_warning_summary: Option<u64>,

    /// Data Units Read (Bytes 47:32):
    /// Number of 512-byte data units read from controller
    /// Reported in thousands (value of 1 = 1,000 units)
    /// Does not include metadata
    pub data_units_read: Option<u64>,

    /// Data Units Written (Bytes 63:48):
    /// Number of 512-byte data units written to controller
    /// Reported in thousands (value of 1 = 1,000 units)
    /// Does not include metadata
    pub data_units_written: Option<u64>,

    /// Host Read Commands (Bytes 79:64):
    /// Number of SMART Host Read Commands completed by the controller
    pub host_read_commands: Option<u64>,

    /// Host Write Commands (Bytes 95:80):
    /// Number of User Data Out Commands completed by the controller
    pub host_write_commands: Option<u64>,

    /// Controller Busy Time (Bytes 111:96):
    /// Amount of time controller is busy with I/O commands
    /// Reported in minutes
    pub controller_busy_time: Option<u64>,

    /// Power Cycles (Bytes 127:112):
    /// Number of power cycles
    pub power_cycles: Option<u64>,

    /// Power On Hours (Bytes 143:128):
    /// Number of power-on hours
    /// May not include time controller was powered in non-operational state
    pub power_on_hours: Option<u64>,

    /// Unsafe Shutdowns / Unexpected Power Losses (Bytes 159:144):
    /// Count of unexpected power losses where controller was not ready
    /// to be powered off or media was not in shutdown state
    pub unsafe_shutdowns: Option<u64>,

    /// Media and Data Integrity Errors (Bytes 175:160):
    /// Number of occurrences where controller detected un-recovered data integrity error
    /// Includes uncorrectable ECC, CRC checksum failure, LBA tag mismatch
    pub media_errors: Option<u64>,

    /// Number of Error Information Log Entries (Bytes 191:176):
    /// Number of Error Information Log Entries over the life of the controller
    pub num_err_log_entries: Option<u64>,

    /// Warning Composite Temperature Time (Bytes 195:192):
    /// Time in minutes that Composite Temperature is >= Warning Threshold
    /// and < Critical Threshold
    pub warning_temp_time: Option<u64>,

    /// Critical Composite Temperature Time (Bytes 199:196):
    /// Time in minutes that Composite Temperature is >= Critical Threshold
    pub critical_comp_time: Option<u64>,

    /// Temperature Sensor 1 (Bytes 201:200):
    /// Current temperature reported by temperature sensor 1 in Kelvins
    pub temperature_sensor_1: Option<u64>,

    /// Temperature Sensor 2 (Bytes 203:202):
    /// Current temperature reported by temperature sensor 2 in Kelvins
    pub temperature_sensor_2: Option<u64>,

    /// Temperature Sensor 3 (Bytes 205:204):
    /// Current temperature reported by temperature sensor 3 in Kelvins
    pub temperature_sensor_3: Option<u64>,

    /// Temperature Sensor 4 (Bytes 207:206):
    /// Current temperature reported by temperature sensor 4 in Kelvins
    pub temperature_sensor_4: Option<u64>,

    /// Temperature Sensor 5 (Bytes 209:208):
    /// Current temperature reported by temperature sensor 5 in Kelvins
    pub temperature_sensor_5: Option<u64>,

    /// Temperature Sensor 6 (Bytes 211:210):
    /// Current temperature reported by temperature sensor 6 in Kelvins
    pub temperature_sensor_6: Option<u64>,

    /// Temperature Sensor 7 (Bytes 213:212):
    /// Current temperature reported by temperature sensor 7 in Kelvins
    pub temperature_sensor_7: Option<u64>,

    /// Temperature Sensor 8 (Bytes 215:214):
    /// Current temperature reported by temperature sensor 8 in Kelvins
    pub temperature_sensor_8: Option<u64>,

    /// Thermal Management Temperature 1 Transition Count (Bytes 219:216):
    /// Number of times controller transitioned to lower power states to reduce
    /// temperature after rising above Thermal Management Temperature 1
    /// Does not wrap after reaching 0xFFFFFFFF
    pub thm_temp1_trans_count: Option<u64>,

    /// Thermal Management Temperature 2 Transition Count (Bytes 223:220):
    /// Number of times controller performed heavy thermal throttling to reduce
    /// temperature after rising above Thermal Management Temperature 2
    /// Does not wrap after reaching 0xFFFFFFFF
    pub thm_temp2_trans_count: Option<u64>,

    /// Total Time For Thermal Management Temperature 1 (Bytes 227:224):
    /// Number of seconds controller spent in lower power states due to
    /// Thermal Management Temperature 1. Reported in seconds
    /// Does not wrap after reaching 0xFFFFFFFF
    pub thm_temp1_total_time: Option<u64>,

    /// Total Time For Thermal Management Temperature 2 (Bytes 231:228):
    /// Number of seconds controller spent performing heavy throttling due to
    /// Thermal Management Temperature 2. Reported in seconds
    /// Does not wrap after reaching 0xFFFFFFFF
    pub thm_temp2_total_time: Option<u64>,
}

// Constructor for NvmesSmartLog
impl NvmesSmartLog {
    pub fn new(nvme_name: String, raw: &nvme_smart_log) -> Self {
        // TODO: Add validation for values from unsafe crate
        Self {
            nvme_name,
            critical_warning: Some(raw.critical_warning as u64),
            temperature: Some(u16::from_le_bytes([raw.temperature[0], raw.temperature[1]]) as u64),
            avail_spare: Some(raw.avail_spare as u64),
            spare_thresh: Some(raw.spare_thresh as u64),
            percent_used: Some(raw.percent_used as u64),
            // NOTE: The linux_nvme_sys crate does not expose byte 06 (Endurance Group Critical
            // Warning Summary) as a separate field. Instead, it's lumped into the rsvd6 reserved
            // byte array. According to the NVMe spec, byte 06 is the endurance group warning field,
            // which corresponds to rsvd6[0].
            endurance_grp_critical_warning_summary: Some(raw.rsvd6[0] as u64),
            data_units_read: Some(u128::from_le_bytes(raw.data_units_read) as u64),
            data_units_written: Some(u128::from_le_bytes(raw.data_units_written) as u64),
            host_read_commands: Some(u128::from_le_bytes(raw.host_reads) as u64),
            host_write_commands: Some(u128::from_le_bytes(raw.host_writes) as u64),
            controller_busy_time: Some(u128::from_le_bytes(raw.ctrl_busy_time) as u64),
            power_cycles: Some(u128::from_le_bytes(raw.power_cycles) as u64),
            power_on_hours: Some(u128::from_le_bytes(raw.power_on_hours) as u64),
            unsafe_shutdowns: Some(u128::from_le_bytes(raw.unsafe_shutdowns) as u64),
            media_errors: Some(u128::from_le_bytes(raw.media_errors) as u64),
            num_err_log_entries: Some(u128::from_le_bytes(raw.num_err_log_entries) as u64),
            warning_temp_time: Some(u32::from(raw.warning_temp_time) as u64),
            critical_comp_time: Some(u32::from(raw.critical_comp_time) as u64),

            // All 8 temperature sensors covered in the specs
            temperature_sensor_1: Some(u16::from(raw.temp_sensor[0]) as u64),
            temperature_sensor_2: Some(u16::from(raw.temp_sensor[1]) as u64),
            temperature_sensor_3: Some(u16::from(raw.temp_sensor[2]) as u64),
            temperature_sensor_4: Some(u16::from(raw.temp_sensor[3]) as u64),
            temperature_sensor_5: Some(u16::from(raw.temp_sensor[4]) as u64),
            temperature_sensor_6: Some(u16::from(raw.temp_sensor[5]) as u64),
            temperature_sensor_7: Some(u16::from(raw.temp_sensor[6]) as u64),
            temperature_sensor_8: Some(u16::from(raw.temp_sensor[7]) as u64),

            thm_temp1_trans_count: Some(u32::from(raw.thm_temp1_trans_count) as u64),
            thm_temp2_trans_count: Some(u32::from(raw.thm_temp2_trans_count) as u64),
            thm_temp1_total_time: Some(u32::from(raw.thm_temp1_total_time) as u64),
            thm_temp2_total_time: Some(u32::from(raw.thm_temp2_total_time) as u64),
        }
    }
}

/// Function to discover controllers exposed on the server.
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

/// Function to extract raw nvme_smart_log from a controller.
/// NOTE - This function is heavily annotated because I was struggling to understand how data is extracted.
pub fn get_nvme_smart_log_raw(dev_path: &str) -> io::Result<nvme_smart_log> {
    let file = OpenOptions::new()
        .read(true)
        .write(true) // Here we need admin permission to send write commands
        .open(dev_path)?; // path would be something like /dev/nvme0

    // This is the raw file descriptor when we make the kernel call. file is Rust's fancy wrapper with safety features.
    let fd = file.as_raw_fd();

    // Effectively memory allocation for the response. nvme_smart_log is defined by the crete,
    // we create a mutable variable for the results and fill it with zeros to then replace.
    // This is unsafe, technically, because zero initialization might not be safe for all the members.
    let mut log: nvme_smart_log = unsafe { zeroed() };

    // log_ptr is the address where the kernel will write the data we want
    let log_ptr = &mut log as *mut nvme_smart_log as u64;
    // log_len is the size we allocate
    let log_len = size_of::<nvme_smart_log>() as u32;

    // From NVMe Base Specification Document:
    // This log page is used to provide SMART and general health information. The information provided is over
    // the life of the controller and is retained across power cycles unless otherwise specified

    let log_id: u8 = 0x02; // SMART/Health Information - Log Page Identifier 02h 
    let numd: u32 = (log_len / 4 - 1).into();
    let cdw10: u32 = (log_id as u32) | (numd << 16);

    let mut cmd: nvme_admin_cmd = unsafe { zeroed() };
    cmd.opcode = nvme_admin_get_log_page as u8;
    // If a namespace identifier other than 0h or FFFFFFFFh is specified by the host,
    // then the controller shall abort the command with a status code of Invalid Field in Command;
    cmd.nsid = 0xFFFF_FFFF;
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

/// Function to collect extracted smart log data.
pub fn collect_smart_log() -> Vec<NvmesSmartLog> {
    let mut results = Vec::new();
    let ctrls = list_nvme_controllers();

    for ctrl in ctrls {
        let dev_path = format!("/dev/{}", ctrl);

        match get_nvme_smart_log_raw(&dev_path) {
            Ok(raw) => {
                let mapped = NvmesSmartLog::new(ctrl.clone(), &raw);
                results.push(mapped);
            }
            Err(e) => {
                eprintln!("Failed to fetch SMART log for {}: {}", dev_path, e);
            }
        }
    }
    results
}
