//! Handles "Abort Transfer" messages.

use std::sync::Mutex;
use message::{BackendServices, Notification};
use handler::{Transfers, HandleMessageParams};

/// Type for "Abort Transfer" messages.
#[derive(RustcDecodable)]
/// The package transfer to abort
pub struct AbortParams;

impl HandleMessageParams for AbortParams {
    fn handle(&self,
              _: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              _: &str, _: &str, _: &str) -> bool {
        let mut transfers = transfers.lock().unwrap();
        transfers.clear();
        true
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
    fn it_removes_a_single_transfer() {
        test_init!();

        let services = Mutex::new(get_empty_backend());

        let prefix = PathPrefix::new();
        let mut transfer = Transfer::new_test(&prefix);
        let package = transfer.randomize(10);

        let transfers = Mutex::new(HashMap::new());
        transfers.lock().unwrap().insert(package.clone(), transfer);

        let abort = AbortParams;
        assert!(abort.handle(&services, &transfers, "", "", ""));
        assert!(transfers.lock().unwrap().is_empty());
    }

    #[test]
    fn it_removes_all_transfers() {
        test_init!();
        let services = Mutex::new(get_empty_backend());
        let prefix = PathPrefix::new();

        let transfers = Mutex::new(HashMap::new());
        for i in 1..20 {
            let mut transfer = Transfer::new_test(&prefix);
            let package = transfer.randomize(i);
            transfers.lock().unwrap().insert(package, transfer);
        }

        let abort = AbortParams;
        assert!(abort.handle(&services, &transfers, "", "", ""));
        assert!(transfers.lock().unwrap().is_empty());
    }
}
