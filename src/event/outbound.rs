use super::UpdateId;

#[derive(RustcDecodable, Debug, Clone)]
pub struct InstalledFirmware {
    pub module: String,
    pub firmware_id: String,
    pub last_modified: u64
}

pub struct InstalledFirmwares(pub Vec<InstalledFirmware>);

#[derive(RustcDecodable, Debug, Clone)]
pub struct InstalledPackage {
    pub package_id: String,
    pub name: String,
    pub description: String,
    pub last_modified: u64
}

pub struct InstalledPackages(pub Vec<InstalledPackage>);

pub struct InstalledSoftware {
    pub packages: InstalledPackages,
    pub firmware: InstalledFirmwares
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct OperationResult {
    pub id: String,
    pub result_code: u32,
    pub result_text: String
}

pub struct OperationResults(pub Vec<OperationResult>);

pub struct UpdateReport {
    pub update_id: String,
    pub operation_results: OperationResults
}

impl UpdateReport {
    pub fn new(id: String, res: OperationResults) -> UpdateReport {
        UpdateReport {
            update_id: id,
            operation_results: res
        }
    }
}

pub enum OutBoundEvent {
    InitiateDownload(UpdateId),
    AbortDownload(UpdateId),
    UpdateReport(UpdateReport)
}
