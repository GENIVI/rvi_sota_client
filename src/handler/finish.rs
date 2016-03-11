//! Handles "Finish Transfer" messages.

use std::sync::Mutex;

use event::UpdateId;
use event::inbound::{InboundEvent, DownloadComplete};
// use message::{PackageId, ServerPackageReport};
use handler::{Error, Result, RemoteServices, HandleMessageParams};
use persistence::Transfers;

/// Type for "Finish Transfer" messages.
#[derive(RustcDecodable)]
pub struct FinishParams {
    /// The package transfer to finalize.
    pub update_id: UpdateId,
    pub signature: String
}

impl HandleMessageParams for FinishParams {
    fn handle(&self,
              _: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>) -> Result {
        let mut transfers = transfers.lock().unwrap();
        let success = transfers.get(&self.update_id).map(|t| {
            t.assemble_package()
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for update_id {}", self.update_id);
            false
        });
        if success {
            transfers.remove(&self.update_id);
            info!("Finished transfer of {}", self.update_id);
            Ok(Some(InboundEvent::DownloadComplete(DownloadComplete {
                update_image: String::new(),
                signature: self.signature.clone()
            })))
        } else {
            /*
        let services = services.lock().unwrap();
            let _ = services.send_package_report(
                ServerPackageReport {
                    package: self.package.clone(),
                    status: false,
                    description: "checksums didn't match".to_string(),
                    vin: services.vin.clone() })
                .map_err(|e| {
                    error!("Error on sending ServerPackageReport: {}", e);
                    Error::SendFailure });
                    */
            Err(Error::UnknownPackage)
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use super::*;
    use test_library::*;

    use rustc_serialize::base64;
    use rustc_serialize::base64::ToBase64;

    use handler::{HandleMessageParams, ChunkParams};
    use persistence::{Transfer, Transfers};

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
            assert!(chunk.handle(&$services, &$transfers).is_ok());
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
            let transfers = Mutex::new(Transfers::new("".to_string()));
            transfers.lock().unwrap().push_test(transfer);
            let services = Mutex::new(get_empty_backend());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers).is_ok());
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
            let transfers = Mutex::new(Transfers::new("".to_string()));
            transfers.lock().unwrap().push_test(transfer);
            let services = Mutex::new(get_empty_backend());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers).is_ok());
            assert!(transfers.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn it_returns_false_on_invalid_transfers() {
        test_init!();
        for i in 1..20 {
            let package = generate_random_package(i);
            let transfers = Mutex::new(Transfers::new("".to_string()));
            let services = Mutex::new(get_empty_backend());

            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers).is_err());
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
            let transfers = Mutex::new(Transfers::new("".to_string()));
            transfers.lock().unwrap().push_test(transfer);
            let services = Mutex::new(get_empty_backend());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: generate_random_package(i) };
            assert!(!finish.handle(&services, &transfers).is_ok());
            assert!(!transfers.lock().unwrap().is_empty());
        }
    }

}
