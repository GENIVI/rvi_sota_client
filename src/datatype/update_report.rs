use rustc_serialize::{Encodable, Encoder};
use std::str::FromStr;

use datatype::{Error, UpdateRequestId};


/// An encodable report of the installation outcome.
#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct UpdateReport {
    pub update_id:         UpdateRequestId,
    pub operation_results: Vec<OperationResult>
}

impl UpdateReport {
    /// Instantiate a new report with a vector of installation outcomes.
    pub fn new(update_id: String, results: Vec<OperationResult>) -> UpdateReport {
        UpdateReport { update_id: update_id, operation_results: results }
    }

    /// Instantiate a new report with a single installation outcome.
    pub fn single(update_id: UpdateRequestId, result_code: UpdateResultCode, result_text: String) -> UpdateReport {
        let result = OperationResult {
            id: update_id.clone(),
            result_code: result_code,
            result_text: result_text
        };
        UpdateReport { update_id: update_id, operation_results: vec![result] }
    }
}

impl Default for UpdateReport {
    fn default() -> Self {
        UpdateReport { update_id: "".to_string(), operation_results: Vec::new() }
    }
}


/// Bind the installation outcome report to a specific device.
#[derive(RustcEncodable, Clone, Debug)]
pub struct DeviceReport<'d, 'r> {
    pub device:        &'d str,
    pub update_report: &'r UpdateReport
}

impl<'d, 'r> DeviceReport<'d, 'r> {
    /// Instantiate a new installation outcome report for a specific device.
    pub fn new(device: &'d str, update_report: &'r UpdateReport) -> DeviceReport<'d, 'r> {
        DeviceReport { device: device, update_report: update_report }
    }
}


/// Enumerate the possible outcomes when trying to install a package.
#[allow(non_camel_case_types)]
#[derive(RustcDecodable, Clone, Debug, PartialEq, Eq)]
pub enum UpdateResultCode {
    /// Operation executed successfully
    OK = 0,
    /// Operation has already been processed
    ALREADY_PROCESSED,
    /// Dependency failure during package install, upgrade, or removal
    DEPENDENCY_FAILURE,
    /// Update image integrity has been compromised
    VALIDATION_FAILED,
    /// Package installation failed
    INSTALL_FAILED,
    /// Package upgrade failed
    UPGRADE_FAILED,
    /// Package removal failed
    REMOVAL_FAILED,
    /// The module loader could not flash its managed module
    FLASH_FAILED,
    /// Partition creation failed
    CREATE_PARTITION_FAILED,
    /// Partition deletion failed
    DELETE_PARTITION_FAILED,
    /// Partition resize failed
    RESIZE_PARTITION_FAILED,
    /// Partition write failed
    WRITE_PARTITION_FAILED,
    /// Partition patching failed
    PATCH_PARTITION_FAILED,
    /// User declined the update
    USER_DECLINED,
    /// Software was blacklisted
    SOFTWARE_BLACKLISTED,
    /// Ran out of disk space
    DISK_FULL,
    /// Software package not found
    NOT_FOUND,
    /// Tried to downgrade to older version
    OLD_VERSION,
    /// SWM Internal integrity error
    INTERNAL_ERROR,
    /// Other error
    GENERAL_ERROR,
}

impl FromStr for UpdateResultCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<UpdateResultCode, Error> {
        match &*s.to_uppercase() {
            "0"  | "OK"                      => Ok(UpdateResultCode::OK),
            "1"  | "ALREADY_PROCESSED"       => Ok(UpdateResultCode::ALREADY_PROCESSED),
            "2"  | "DEPENDENCY_FAILURE"      => Ok(UpdateResultCode::DEPENDENCY_FAILURE),
            "3"  | "VALIDATION_FAILED"       => Ok(UpdateResultCode::VALIDATION_FAILED),
            "4"  | "INSTALL_FAILED"          => Ok(UpdateResultCode::INSTALL_FAILED),
            "5"  | "UPGRADE_FAILED"          => Ok(UpdateResultCode::UPGRADE_FAILED),
            "6"  | "REMOVAL_FAILED"          => Ok(UpdateResultCode::REMOVAL_FAILED),
            "7"  | "FLASH_FAILED"            => Ok(UpdateResultCode::FLASH_FAILED),
            "8"  | "CREATE_PARTITION_FAILED" => Ok(UpdateResultCode::CREATE_PARTITION_FAILED),
            "9"  | "DELETE_PARTITION_FAILED" => Ok(UpdateResultCode::DELETE_PARTITION_FAILED),
            "10" | "RESIZE_PARTITION_FAILED" => Ok(UpdateResultCode::RESIZE_PARTITION_FAILED),
            "11" | "WRITE_PARTITION_FAILED"  => Ok(UpdateResultCode::WRITE_PARTITION_FAILED),
            "12" | "PATCH_PARTITION_FAILED"  => Ok(UpdateResultCode::PATCH_PARTITION_FAILED),
            "13" | "USER_DECLINED"           => Ok(UpdateResultCode::USER_DECLINED),
            "14" | "SOFTWARE_BLACKLISTED"    => Ok(UpdateResultCode::SOFTWARE_BLACKLISTED),
            "15" | "DISK_FULL"               => Ok(UpdateResultCode::DISK_FULL),
            "16" | "NOT_FOUND"               => Ok(UpdateResultCode::NOT_FOUND),
            "17" | "OLD_VERSION"             => Ok(UpdateResultCode::OLD_VERSION),
            "18" | "INTERNAL_ERROR"          => Ok(UpdateResultCode::INTERNAL_ERROR),
            "19" | "GENERAL_ERROR"           => Ok(UpdateResultCode::GENERAL_ERROR),
            _ => Err(Error::Parse(format!("unknown UpdateResultCode: {}", s)))
        }
    }
}

impl Encodable for UpdateResultCode {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u64(self.clone() as u64)
    }
}


/// An encodable response of the installation outcome for a particular update ID.
#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct OperationResult {
    pub id:          String,
    pub result_code: UpdateResultCode,
    pub result_text: String,
}


/// Encapsulates a single firmware installed on the device.
#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct InstalledFirmware {
    pub module:        String,
    pub firmware_id:   String,
    pub last_modified: u64
}


/// Encapsulates a single package installed on the device.
#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct InstalledPackage {
    pub package_id:    String,
    pub name:          String,
    pub description:   String,
    pub last_modified: u64
}


/// An encodable list of packages and firmwares to send to RVI.
#[derive(RustcDecodable, RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct InstalledSoftware {
    pub packages:  Vec<InstalledPackage>,
    pub firmwares: Vec<InstalledFirmware>
}

impl InstalledSoftware {
    /// Instantiate a new list of the software installed on the device.
    pub fn new(packages: Vec<InstalledPackage>, firmwares: Vec<InstalledFirmware>) -> InstalledSoftware {
        InstalledSoftware { packages: packages, firmwares: firmwares }
    }
}

impl Default for InstalledSoftware {
    fn default() -> Self {
        InstalledSoftware { packages: Vec::new(), firmwares: Vec::new() }
    }
}
