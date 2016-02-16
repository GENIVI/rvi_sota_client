//! Handles "Abort Transfer" messages.

use std::sync::Mutex;
use message::BackendServices;
use handler::{Result, HandleMessageParams};
use persistence::Transfers;

/// Type for "Abort Transfer" messages.
#[derive(RustcDecodable)]
/// The package transfer to abort
pub struct AbortParams;

impl HandleMessageParams for AbortParams {
    fn handle(&self,
              _: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              _: &str,
              _: &str) -> Result {
        let mut transfers = transfers.lock().unwrap();
        transfers.clear();
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_library::*;

    use std::sync::Mutex;

    use handler::HandleMessageParams;
    use persistence::{Transfer, Transfers};

    #[test]
    fn it_removes_a_single_transfer() {
        test_init!();

        let services = Mutex::new(get_empty_backend());

        let prefix = PathPrefix::new();
        let mut transfer = Transfer::new_test(&prefix);
        transfer.randomize(10);

        let transfers = Mutex::new(Transfers::new(prefix.to_string()));
        transfers.lock().unwrap().push_test(transfer);

        let abort = AbortParams;
        assert!(abort.handle(&services, &transfers, "", "").is_ok());
        assert!(transfers.lock().unwrap().is_empty());
    }

    #[test]
    fn it_removes_all_transfers() {
        test_init!();
        let services = Mutex::new(get_empty_backend());
        let prefix = PathPrefix::new();

        let transfers = Mutex::new(Transfers::new(prefix.to_string()));
        for i in 1..20 {
            let mut transfer = Transfer::new_test(&prefix);
            transfer.randomize(i);
            transfers.lock().unwrap().push_test(transfer);
        }

        let abort = AbortParams;
        assert!(abort.handle(&services, &transfers, "", "").is_ok());
        assert!(transfers.lock().unwrap().is_empty());
    }
}
