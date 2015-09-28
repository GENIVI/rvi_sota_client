use std::sync::Mutex;
use std::collections::HashMap;

use message::{BackendServices, PackageId, UserMessage};
use handler::HandleMessageParams;
use persistence::Transfer;

#[derive(RustcDecodable)]
pub struct FinishParams {
    pub package: PackageId
}

impl HandleMessageParams for FinishParams {
    fn handle(&self,
              _: &Mutex<BackendServices>,
              transfers: &Mutex<HashMap<PackageId, Transfer>>,
              _: &str, _: &str, _: &str) -> bool {
        let mut transfers = transfers.lock().unwrap();
        let success = transfers.get(&self.package).map(|t| {
            t.assemble_package() && t.install_package()
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for package {}", self.package);
            false
        });
        if success {
            transfers.remove(&self.package);
            info!("Finished transfer of {}", self.package);
        }
        success
    }

    fn get_message(&self) -> Option<UserMessage> { None }
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
            assert!(chunk.handle(&$services, &$transfers, "ignored", "", ""));
        }}
    }

    #[test]
    fn it_returns_true_on_existing_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new(&prefix);
            transfer.checksum =
                "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83".to_string();
            let package = transfer.randomize(i);
            let transfers = Mutex::new(HashMap::new());
            transfers.lock().unwrap().insert(package.clone(), transfer);
            let services = Mutex::new(BackendServices::new());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers, "ignored", "", ""));
        }
    }

    #[test]
    fn it_removes_existing_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new(&prefix);
            transfer.checksum =
                "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83".to_string();
            let package = transfer.randomize(i);
            let transfers = Mutex::new(HashMap::new());
            transfers.lock().unwrap().insert(package.clone(), transfer);
            let services = Mutex::new(BackendServices::new());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: package.clone() };
            assert!(finish.handle(&services, &transfers, "ignored", "", ""));
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
            assert!(!finish.handle(&services, &transfers, "ignored", "", ""));
        }
    }

    #[test]
    fn it_does_not_touch_transfers_on_invalid_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new(&prefix);
            transfer.checksum =
                "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83".to_string();
            let package = transfer.randomize(i);
            let transfers = Mutex::new(HashMap::new());
            transfers.lock().unwrap().insert(package.clone(), transfer);
            let services = Mutex::new(BackendServices::new());

            assert_data_written!(package, services, transfers);
            let finish = FinishParams { package: generate_random_package(i) };
            assert!(!finish.handle(&services, &transfers, "ignored", "", ""));
            assert!(!transfers.lock().unwrap().is_empty());
        }
    }
}
