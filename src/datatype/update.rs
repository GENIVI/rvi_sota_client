use rustc_serialize::{Encodable, Encoder};

use super::UpdateRequestId;


#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct UpdateReport {
    pub update_id: UpdateRequestId,
    pub results:   Vec<OperationResult>
}

impl UpdateReport {
    pub fn new(update_id: String, results: Vec<OperationResult>) -> UpdateReport {
        UpdateReport { update_id: update_id, results: results }
    }

    pub fn single(update_id: UpdateRequestId, code: UpdateResultCode, output: String) -> UpdateReport {
        let result = OperationResult { id: update_id.clone(), code: code, output: output };
        UpdateReport { update_id: update_id, results: vec![result] }
    }
}


#[derive(RustcEncodable, Clone, Debug)]
pub struct DeviceReport<'u, 'r> {
    pub uuid:   &'u str,
    pub report: &'r UpdateReport
}

impl<'u, 'r> DeviceReport<'u, 'r> {
    pub fn new(uuid: &'u str, report: &'r UpdateReport) -> DeviceReport<'u, 'r> {
        DeviceReport { uuid: &uuid, report: &report }
    }
}


#[allow(non_camel_case_types)]
#[derive(RustcDecodable, Clone, Debug, PartialEq, Eq)]
pub enum UpdateResultCode {
    OK = 0,                  // Operation executed successfully
    ALREADY_PROCESSED,       // Operation has already been processed
    DEPENDENCY_FAILURE,      // Dependency failure during package install, upgrade, or removal
    VALIDATION_FAILED,       // Update image integrity has been compromised
    INSTALL_FAILED,          // Package installation failed
    UPGRADE_FAILED,          // Package upgrade failed
    REMOVAL_FAILED,          // Package removal failed
    FLASH_FAILED,            // The module loader could not flash its managed module
    CREATE_PARTITION_FAILED, // Partition creation failed
    DELETE_PARTITION_FAILED, // Partition deletion failed
    RESIZE_PARTITION_FAILED, // Partition resize failed
    WRITE_PARTITION_FAILED,  // Partition write failed
    PATCH_PARTITION_FAILED,  // Partition patching failed
    USER_DECLINED,           // User declined the update
    SOFTWARE_BLACKLISTED,    // Software was blacklisted
    DISK_FULL,               // Ran out of disk space
    NOT_FOUND,               // Software package not found
    OLD_VERSION,             // Tried to downgrade to older version
    INTERNAL_ERROR,          // SWM Internal integrity error
    GENERAL_ERROR,           // Other error
}

impl Encodable for UpdateResultCode {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u64(self.clone() as u64)
    }
}


#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct OperationResult {
    pub id:     String,
    pub code:   UpdateResultCode,
    pub output: String,
}


#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct InstalledFirmware {
    pub module:        String,
    pub firmware_id:   String,
    pub last_modified: u64
}


#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct InstalledPackage {
    pub package_id:    String,
    pub name:          String,
    pub description:   String,
    pub last_modified: u64
}


#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct InstalledSoftware {
    pub packages:  Vec<InstalledPackage>,
    pub firmwares: Vec<InstalledFirmware>
}

impl InstalledSoftware {
    pub fn new(packages: Vec<InstalledPackage>, firmwares: Vec<InstalledFirmware>) -> InstalledSoftware {
        InstalledSoftware { packages: packages, firmwares: firmwares }
    }
}
