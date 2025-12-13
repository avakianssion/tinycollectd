// src/collector/nvme.rs
//! NVMe SMART collection via linux_nvme_sys.

use nvme_cli_sys::{
    nvme_admin_cmd, nvme_admin_opcode::nvme_admin_get_log_page,
    nvme_admin_opcode::nvme_admin_identify, nvme_id_ctrl, nvme_smart_log,nvme_id_power_state
};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io;
use std::mem::{size_of, zeroed};
use std::os::unix::io::AsRawFd;

#[derive(Debug, Serialize)]
pub struct NvmesIdCtrl {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// PCI Vendor ID of the controller.
    pub vid: u16,

    /// PCI Subsystem Vendor ID.
    pub ssvid: u16,

    /// Serial Number (ASCII, space padded).
    pub sn: String,

    /// Model Number (ASCII, space padded).
    pub mn: String,

    /// Firmware Revision (ASCII, space padded).
    pub fr: String,

    /// Recommended Arbitration Burst.
    /// Hint to host for arbitration burst size when using weighted round-robin arbitration.
    pub rab: u8,

    /// IEEE OUI Identifier (3 bytes) for the vendor (Organizationally Unique Identifier).
    pub ieee: [u8; 3],

    /// Controller Multi-Path I/O and Namespace Sharing Capabilities (bitfield).
    /// Indicates multipath / shared namespaces capabilities.
    pub cmic: u8,

    /// Maximum Data Transfer Size (MDTS).
    /// Expressed as a power-of-two multiple of the minimum memory page size (MPSMIN).
    /// Effective max transfer = (2^mdts) * (minimum page size).
    pub mdts: u8,

    /// Controller ID (CNTLID) assigned by the controller.
    pub cntlid: u16,

    /// Version (VER) of the NVMe specification the controller complies with.
    pub ver: u32,

    /// RTD3 Resume Latency (microseconds).
    pub rtd3r_us: u32,

    /// RTD3 Entry Latency (microseconds).
    pub rtd3e_us: u32,

    /// Optional Asynchronous Events Supported (bitfield).
    pub oaes: u32,

    /// Controller Attributes (bitfield).
    pub ctratt: u32,

    /// Read Recovery Levels Supported (bitfield / encoded).
    pub rrls: u16,

    /// Controller Type (encoded).
    pub cntrltype: u8,

    /// FRU GUID / Field Replaceable Unit GUID.
    pub fguid: [u8; 16],

    /// Command Retry Delay Time 1.
    pub crdt1: u16,

    /// Command Retry Delay Time 2.
    pub crdt2: u16,

    /// Command Retry Delay Time 3.
    pub crdt3: u16,

    /// NVM Subsystem Report (bitfield/encoded).
    pub nvmsr: u8,

    /// VPD Write Cycle Information (bitfield/encoded).
    pub vwci: u8,

    /// Management Endpoint Capabilities (bitfield/encoded).
    pub mec: u8,

    /// Optional Admin Command Support (bitfield).
    pub oacs: u16,

    /// Abort Command Limit.
    pub acl: u8,

    /// Asynchronous Event Request Limit.
    pub aerl: u8,

    /// Firmware Updates (bitfield).
    pub frmw: u8,

    /// Log Page Attributes (bitfield).
    pub lpa: u8,

    /// Error Log Page Entries (0-based).
    pub elpe: u8,

    /// Number of Power States Supported minus 1.
    pub npss: u8,

    /// Admin Vendor Specific Command Configuration.
    pub avscc: u8,

    /// Autonomous Power State Transition Attributes.
    pub apsta: u8,

    /// Warning Composite Temperature Threshold (Kelvin).
    pub wctemp_k: u16,

    /// Critical Composite Temperature Threshold (Kelvin).
    pub cctemp_k: u16,

    /// Maximum Time for Firmware Activation.
    pub mtfa: u16,

    /// Host Memory Buffer Preferred Size (bytes).
    pub hmpre: u32,

    /// Host Memory Buffer Minimum Size (bytes).
    pub hmmin: u32,

    /// Total NVM Capacity (bytes).
    pub tnvmcap_bytes: u128,

    /// Unallocated NVM Capacity (bytes).
    pub unvmcap_bytes: u128,

    /// Replay Protected Memory Block Support (bitfield).
    pub rpmbs: u32,

    /// Extended Device Self-test Time (minutes).
    pub edstt: u16,

    /// Device Self-test Options (bitfield).
    pub dsto: u8,

    /// Firmware Update Granularity.
    pub fwug: u8,

    /// Keep Alive Support.
    pub kas: u16,

    /// Host Controlled Thermal Management Attributes.
    pub hctma: u16,

    /// Minimum Thermal Management Temperature (Kelvin).
    pub mntmt_k: u16,

    /// Maximum Thermal Management Temperature (Kelvin).
    pub mxtmt_k: u16,

    /// Sanitize Capabilities (bitfield).
    pub sanicap: u32,

    /// Host Memory Buffer Minimum Descriptor Entry Size.
    pub hmminds: u32,

    /// Host Memory Buffer Maximum Descriptor Entries.
    pub hmmaxd: u16,

    /// NVM Set Identifier Maximum.
    pub nsetidmax: u16,

    /// Endurance Group Identifier Maximum.
    pub endgidmax: u16,

    /// ANA Transition Time.
    pub anatt: u8,

    /// ANA Capabilities.
    pub anacap: u8,

    /// ANA Group Identifier Maximum.
    pub anagrpmax: u32,

    /// Number of ANA Group Identifiers.
    pub nanagrpid: u32,

    /// Persistent Event Log Size (bytes).
    pub pels: u32,

    /// Domain Identifier.
    pub domainid: u16,

    /// Maximum Endurance Group Capacity (bytes).
    pub megcap_bytes: u128,

    /// Submission Queue Entry Size encoding.
    pub sqes: u8,

    /// Completion Queue Entry Size encoding.
    pub cqes: u8,

    /// Maximum Outstanding Commands.
    pub maxcmd: u16,

    /// Number of Namespaces.
    pub nn: u32,

    /// Optional NVM Command Support.
    pub oncs: u16,

    /// Fused Operation Support.
    pub fuses: u16,

    /// Format NVM Attributes.
    pub fna: u8,

    /// Volatile Write Cache.
    pub vwc: u8,

    /// Atomic Write Unit Normal (logical blocks).
    pub awun: u16,

    /// Atomic Write Unit Power Fail (logical blocks).
    pub awupf: u16,

    /// Vendor Specific Command Configuration.
    pub icsvscc: u8,

    /// Namespace Write Protection Capabilities.
    pub nwpc: u8,

    /// Atomic Compare & Write Unit (logical blocks).
    pub acwu: u16,

    /// Optional Copy Formats Supported.
    pub ocfs: u16,

    /// SGL Support.
    pub sgls: u32,

    /// Maximum Number of Allowed Namespaces.
    pub mnan: u32,

    /// Maximum Capacity of NVM Area.
    pub maxcna: u32,

    /// Subsystem NQN (ASCII).
    pub subnqn: String,

    /// I/O Command Capsule Supported Size.
    pub ioccsz: u32,

    /// I/O Response Capsule Supported Size.
    pub iorcsz: u32,

    /// In Capsule Data Offset.
    pub icdoff: u16,

    /// Fabric Controller Attributes.
    pub fcatt: u8,

    /// Management Service Data Block Descriptor.
    pub msdbd: u8,

    /// Optional Fabric Commands Support.
    pub ofcs: u16,

    /// Power State Descriptors.
    pub psd: [nvme_id_power_state; 32],

    /// Vendor Specific area (1024 bytes).
    pub vs: [u8; 1024],
}


/// Constructor for NvmesIdCtrl
impl NvmesIdCtrl {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {}
    }
}

/// Function to extract raw nvme_id_ctrl using the Identify admin command
pub fn get_nvme_id_ctrl_raw(dev_path: &str) -> io::Result<nvme_id_ctrl> {
    let file = OpenOptions::new()
        .read(true)
        .write(true) // Here we need admin permission to send write commands
        .open(dev_path)?; // path would be something like /dev/nvme0

    let fd = file.as_raw_fd();

    // Identify Controller payload is 4096 bytes based on the C bindings in the nvme_cli_sys crate.
    // If nvme_id_ctrl from your crate is exactly 4096, great.
    // If it's smaller, you should use a [u8; 4096] buffer instead.
    let mut id: nvme_id_ctrl = unsafe { zeroed() };

    let id_ptr = &mut id as *mut nvme_id_ctrl as u64;
    let id_len = size_of::<nvme_id_ctrl>() as u32;

    let cns: u8 = 0x01; // Identify Controller
    let cntlid: u16 = 0x0000; // usually 0
    let cdw10: u32 = (cns as u32) | ((cntlid as u32) << 16);

    let mut cmd: nvme_admin_cmd = unsafe { zeroed() };
    cmd.opcode = nvme_admin_identify as u8; // Identify (0x06)
    cmd.nsid = 0x0000_0000;
    cmd.addr = id_ptr;
    cmd.data_len = id_len;
    cmd.cdw10 = cdw10;
    cmd.cdw11 = 0;
    cmd.timeout_ms = 1000;

    let ret = unsafe { nvme_cli_sys::nvme_ioctl_admin_cmd(fd, &mut cmd) };

    match ret {
        Ok(status) if status == 0 => Ok(id),
        Ok(status) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("NVMe admin command failed, status={:#x}", status),
        )),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
    }
}

#[derive(Debug, Serialize)]
pub struct NvmesSmartLog {
    /// NVMe device name (e.g., "nvme0")
    /// Potential issue - we use u64 for all values in the struct.
    /// If a drive runs long enough or has crazy write workload, the 128-bit SMART counters might
    /// exceed 2^64-1 so we would likely end up truncating data.
    /// TODO - consider changing u64 to u128.
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
        Self {
            nvme_name,
            critical_warning: Some(raw.critical_warning as u64),
            temperature: Some(u16::from_le_bytes([raw.temperature[0], raw.temperature[1]]) as u64),
            avail_spare: Some(raw.avail_spare as u64),
            spare_thresh: Some(raw.spare_thresh as u64),
            percent_used: Some(raw.percent_used as u64),
            endurance_grp_critical_warning_summary: Some(raw.endu_grp_crit_warn_sumry as u64),
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

/// Function to extract raw nvme_smart_log.
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

    let ret = unsafe { nvme_cli_sys::nvme_ioctl_admin_cmd(fd, &mut cmd) };

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
