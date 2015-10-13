use std::sync::Mutex;
use message::{BackendServices, PackageId, Notification};
use handler::{Transfers, HandleMessageParams};

#[derive(RustcDecodable)]
pub struct AbortParams {
    pub package: PackageId,
}

impl HandleMessageParams for AbortParams {
    fn handle(&self,
              _: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              _: &str, _: &str, _: &str) -> bool {
        let mut transfers = transfers.lock().unwrap();

        match transfers.remove(&self.package) {
            Some(..) => {
                info!("Transfer for package {} aborted", self.package);
                true
            },
            None => {
                error!("No transfer for package {}, ignoring abort",
                       self.package);
                false
            }
        }
    }

    fn get_message(&self) -> Option<Notification> { None }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_library::*;

    use std::sync::Mutex;
    use std::collections::HashMap;

    use handler::HandleMessageParams;
    use persistence::Transfer;

    #[test]
    fn it_removes_existing_transfers() {
        test_init!();

        let services = Mutex::new(get_empty_backend());

        let prefix = PathPrefix::new();
        let mut transfer = Transfer::new_test(&prefix);
        let package = transfer.randomize(10);

        let transfers = Mutex::new(HashMap::new());
        transfers.lock().unwrap().insert(package.clone(), transfer);

        let abort = AbortParams {
            package: package
        };

        assert!(abort.handle(&services, &transfers, "", "", ""));
        assert!(transfers.lock().unwrap().is_empty());
    }

    #[test]
    fn it_doesnt_do_anything_for_nonexistent_transfers() {
        let services = Mutex::new(get_empty_backend());

        let prefix = PathPrefix::new();
        let mut transfer = Transfer::new_test(&prefix);
        let package = transfer.randomize(10);

        let transfers = Mutex::new(HashMap::new());
        transfers.lock().unwrap().insert(package, transfer);

        let abort = AbortParams {
            package: generate_random_package(9)
        };

        assert!(! abort.handle(&services, &transfers, "", "", ""));
        assert!(! transfers.lock().unwrap().is_empty());
    }
}
