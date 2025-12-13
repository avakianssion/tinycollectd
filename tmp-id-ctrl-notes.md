# Notes on how to organzie id ctrl struct. 

It makes sense to break out this massive struct into smaller chunks for ease of use.
Building out a massive struct like this in application layer will be pointless and 
expensive.

The massive sruct defined in the FFI is this:

`
pub struct nvme_id_ctrl {
    pub vid: __le16,
    pub ssvid: __le16,
    pub sn: [::std::os::raw::c_char; 20usize],
    pub mn: [::std::os::raw::c_char; 40usize],
    pub fr: [::std::os::raw::c_char; 8usize],
    pub rab: __u8,
    pub ieee: [__u8; 3usize],
    pub cmic: __u8,
    pub mdts: __u8,
    pub cntlid: __le16,
    pub ver: __le32,
    pub rtd3r: __le32,
    pub rtd3e: __le32,
    pub oaes: __le32,
    pub ctratt: __le32,
    pub rrls: __le16,
    pub rsvd102: [__u8; 9usize],
    pub cntrltype: __u8,
    pub fguid: [::std::os::raw::c_char; 16usize],
    pub crdt1: __le16,
    pub crdt2: __le16,
    pub crdt3: __le16,
    pub rsvd134: [__u8; 119usize],
    pub nvmsr: __u8,
    pub vwci: __u8,
    pub mec: __u8,
    pub oacs: __le16,
    pub acl: __u8,
    pub aerl: __u8,
    pub frmw: __u8,
    pub lpa: __u8,
    pub elpe: __u8,
    pub npss: __u8,
    pub avscc: __u8,
    pub apsta: __u8,
    pub wctemp: __le16,
    pub cctemp: __le16,
    pub mtfa: __le16,
    pub hmpre: __le32,
    pub hmmin: __le32,
    pub tnvmcap: [__u8; 16usize],
    pub unvmcap: [__u8; 16usize],
    pub rpmbs: __le32,
    pub edstt: __le16,
    pub dsto: __u8,
    pub fwug: __u8,
    pub kas: __le16,
    pub hctma: __le16,
    pub mntmt: __le16,
    pub mxtmt: __le16,
    pub sanicap: __le32,
    pub hmminds: __le32,
    pub hmmaxd: __le16,
    pub nsetidmax: __le16,
    pub endgidmax: __le16,
    pub anatt: __u8,
    pub anacap: __u8,
    pub anagrpmax: __le32,
    pub nanagrpid: __le32,
    pub pels: __le32,
    pub domainid: __le16,
    pub rsvd358: [__u8; 10usize],
    pub megcap: [__u8; 16usize],
    pub rsvd384: [__u8; 128usize],
    pub sqes: __u8,
    pub cqes: __u8,
    pub maxcmd: __le16,
    pub nn: __le32,
    pub oncs: __le16,
    pub fuses: __le16,
    pub fna: __u8,
    pub vwc: __u8,
    pub awun: __le16,
    pub awupf: __le16,
    pub icsvscc: __u8,
    pub nwpc: __u8,
    pub acwu: __le16,
    pub ocfs: __le16,
    pub sgls: __le32,
    pub mnan: __le32,
    pub maxdna: [__u8; 16usize],
    pub maxcna: __le32,
    pub rsvd564: [__u8; 204usize],
    pub subnqn: [::std::os::raw::c_char; 256usize],
    pub rsvd1024: [__u8; 768usize],
    pub ioccsz: __le32,
    pub iorcsz: __le32,
    pub icdoff: __le16,
    pub fcatt: __u8,
    pub msdbd: __u8,
    pub ofcs: __le16,
    pub rsvd1806: [__u8; 242usize],
    pub psd: [nvme_id_power_state; 32usize],
    pub vs: [__u8; 1024usize],
}
`

## Tentetive organization schema: 

`
#[derive(Serialize)]
pub struct CtrlIdentity {
  nvme_name: String,          // "nvme0"
  vid: u16,
  ssvid: u16,
  serial_number: String,
  model_number: String,
  firmware_rev: String,
  ieee_oui: [u8; 3],
  cntlid: u16,
  ver: u32,
  subnqn: String,
  fguid: [u8; 16],            // or Option<[u8;16]> if all-zero => None (policy)
}

#[derive(Serialize)]
pub struct CtrlCapacity {
  nvme_name: String,
  total_nvm_bytes: u128,         // tnvmcap
  unallocated_nvm_bytes: u128,   // unvmcap
  max_endurance_group_bytes: u128, // megcap
}

#[derive(Serialize)]
pub struct CtrlCapabilities {
  nvme_name: String,
  oacs: u16,
  oncs: u16,
  lpa: u8,
  ctratt: u32,
  oaes: u32,
  sanicap: u32,
  sgls: u32,
  vwc: u8,
  fna: u8,

  anacap: u8,
  anatt: u8,
  anagrpmax: u32,
  nanagrpid: u32,
}

#[derive(Serialize)]
pub struct CtrlLimits {
  nvme_name: String,
  mdts: u8,
  sqes: u8,
  cqes: u8,
  maxcmd: u16,
  nn: u32,
}

#[derive(Serialize)]
pub struct CtrlThermals {
  nvme_name: String,
  wctemp_k: u16,
  cctemp_k: u16,
  mntmt_k: u16,
  mxtmt_k: u16,
}
`

