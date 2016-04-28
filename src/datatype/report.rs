use rustc_serialize::{Encodable, Encoder};
use super::UpdateRequestId;

#[derive(RustcEncodable, Clone, Debug)]
pub struct UpdateReportWithVin<'a, 'b> {
    vin: &'a str,
    update_report: &'b UpdateReport
}

impl<'a, 'b> UpdateReportWithVin<'a, 'b> {
    pub fn new(vin: &'a str, update_report: &'b UpdateReport) -> UpdateReportWithVin<'a, 'b> {
        UpdateReportWithVin { vin: &vin, update_report: &update_report }
    }
}

#[derive(RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct OperationResult {
    pub id: String,
    pub result_code: UpdateResultCode,
    pub result_text: String,
}

#[derive(RustcEncodable, Clone, Debug, PartialEq, Eq)]
pub struct UpdateReport {
    pub update_id: UpdateRequestId,
    pub operation_results: Vec<OperationResult>
}

impl UpdateReport {
    pub fn new(update_id: UpdateRequestId, result_code: UpdateResultCode, result_text: String) -> UpdateReport {
        UpdateReport { update_id: update_id.clone(),
                        operation_results: vec![OperationResult { id: update_id,
                                                                  result_code: result_code,
                                                                  result_text: result_text }] }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateResultCode {
  // Operation executed successfully
  OK = 0,

  // Operation has already been processed
  ALREADY_PROCESSED,

  // Dependency failure during package install, upgrade, or removal
  DEPENDENCY_FAILURE,

  // Update image integrity has been compromised
  VALIDATION_FAILED,

  // Package installation failed
  INSTALL_FAILED,

  // Package upgrade failed
  UPGRADE_FAILED,

  // Package removal failed
  REMOVAL_FAILED,

  // The module loader could not flash its managed module
  FLASH_FAILED,

  // Partition creation failed
  CREATE_PARTITION_FAILED,

  // Partition deletion failed
  DELETE_PARTITION_FAILED,

  // Partition resize failed
  RESIZE_PARTITION_FAILED,

  // Partition write failed
  WRITE_PARTITION_FAILED,

  // Partition patching failed
  PATCH_PARTITION_FAILED,

  // User declined the update
  USER_DECLINED,

  // Software was blacklisted
  SOFTWARE_BLACKLISTED,

  // Ran out of disk space
  DISK_FULL,

  // Software package not found
  NOT_FOUND,

  // Tried to downgrade to older version
  OLD_VERSION,

  // SWM Internal integrity error
  INTERNAL_ERROR,

  // Other error
  GENERAL_ERROR,
}

impl Encodable for UpdateResultCode {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u64(self.clone() as u64)
    }
}

#[cfg(test)]
mod tests {
    use rustc_serialize::json;

    use super::*;

    fn test_report() -> UpdateReport {
        UpdateReport::new("requestid".to_string(), UpdateResultCode::OK, "result text".to_string())
    }

    #[test]
    fn test_serialization() {
        assert_eq!(r#"{"update_id":"requestid","operation_results":[{"id":"requestid","result_code":0,"result_text":"result text"}]}"#.to_string(),
                   json::encode(&test_report()).unwrap());
    }
}
