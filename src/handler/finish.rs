//! Handles "Finish Transfer" messages.

use std::sync::Mutex;

#[cfg(not(test))] use rvi::send_message;

use message::{BackendServices, PackageId, Notification, ServerPackageReport};
use handler::{Result, Transfers, HandleMessageParams};

/// Type for "Finish Transfer" messages.
#[derive(RustcDecodable)]
pub struct FinishParams {
    /// The package transfer to finalize.
    pub package: PackageId
}

impl HandleMessageParams for FinishParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              rvi_url: &str, vin: &str, _: &str) -> Result {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();
        let success = transfers.get(&self.package).map(|t| {
            t.assemble_package()
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for package {}", self.package);
            false
        });
        if success {
            transfers.remove(&self.package);
            info!("Finished transfer of {}", self.package);
            Ok(Some(Notification::Finish(self.package.clone())))
        } else {
            let _ = send_message(rvi_url,
                                 ServerPackageReport {
                                     package: self.package.clone(),
                                     status: false,
                                     description: "checksums didn't match".to_string(),
                                     vin: vin.to_string()
                                 }, &services.report)
            .map_err(|e| { error!("Error on sending ServerPackageReport: {}", e); false });
            Err(false)
        }
    }
}

#[cfg(test)]
use std::result;

#[cfg(test)]
fn send_message(url: &str, chunks: ServerPackageReport, report: &str)
    -> result::Result<bool, bool> {
    trace!("Would send checksum failure for {}, to {} on {}",
           chunks.package, report, url);
    Ok(true)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use test_library::*;

    use rustc_serialize::base64;
    use rustc_serialize::base64::ToBase64;

    use handler::{HandleMessageParams, ChunkParams};
    use message::BackendServices;
    use persistence::Transfer;

    macro_rules! assert_data_written {
        ($package:ident, $services:ident, $transfers:ident) => {{
            let msg = "test\n".to_string();
            let b64_msg = msg.as_bytes().to_base64(
                base64::Config {
                    char_set: base64::CharacterSet::UrlSafe,
                    newline: base64::Newline::LF,
                    pad: true,
                    line_length: None
                });
            let chunk = ChunkParams {
                bytes: b64_msg,
                index: 1,
                package: $package.clone()
            };
            assert!(chunk.handle(&$services, &$transfers, "ignored", "", "").is_ok());
        }}
    }

    #[test]
    fn it_returns_true_on_existing_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new_test(&prefix);
            transfer.checksum =
                "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83".to_string();
            let package = transfer.randomize(i);
            let transfers = Mutex::new(HashMap::new());
            transfers.lock().unwrap().insert(package.clone(), transfer);
            let services = Mutex::new(BackendServices::new());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers, "ignored", "", "").is_ok());
        }
    }

    #[test]
    fn it_removes_existing_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new_test(&prefix);
            transfer.checksum =
                "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83".to_string();
            let package = transfer.randomize(i);
            let transfers = Mutex::new(HashMap::new());
            transfers.lock().unwrap().insert(package.clone(), transfer);
            let services = Mutex::new(BackendServices::new());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers, "ignored", "", "").is_ok());
            assert!(transfers.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn it_returns_false_on_invalid_transfers() {
        test_init!();
        for i in 1..20 {
            let package = generate_random_package(i);
            let transfers = Mutex::new(HashMap::new());
            let services = Mutex::new(BackendServices::new());

            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers, "ignored", "", "").is_err());
        }
    }

    #[test]
    fn it_does_not_touch_transfers_on_invalid_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new_test(&prefix);
            transfer.checksum =
                "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83".to_string();
            let package = transfer.randomize(i);
            let transfers = Mutex::new(HashMap::new());
            transfers.lock().unwrap().insert(package.clone(), transfer);
            let services = Mutex::new(BackendServices::new());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: generate_random_package(i) };
            assert!(!finish.handle(&services, &transfers, "ignored", "", "").is_ok());
            assert!(!transfers.lock().unwrap().is_empty());
        }
    }
}
