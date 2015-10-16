use std::sync::Mutex;

#[cfg(not(test))] use rvi::send_message;

use message::{BackendServices, PackageId, ChunkReceived, Notification};
use handler::{Transfers, HandleMessageParams};

#[derive(RustcDecodable)]
pub struct ChunkParams {
    pub bytes: String,
    pub index: u64,
    pub package: PackageId
}

impl HandleMessageParams for ChunkParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              rvi_url: &str, vin: &str, _: &str) -> bool {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();
        transfers.get_mut(&self.package).map(|t| {
            if t.write_chunk(&self.bytes, self.index) {
                info!("Wrote chunk {} for package {}", self.index, self.package);
                try_or!(send_message(rvi_url,
                                     ChunkReceived {
                                         package: self.package.clone(),
                                         chunks: t.transferred_chunks.clone(),
                                         vin: vin.to_string()
                                     },
                                     &services.ack), return false);
                true
            } else {
                false
            }
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for package {}", self.package);
            false
        })
    }

    fn get_message(&self) -> Option<Notification> { None }
}

#[cfg(test)]
fn send_message(url: &str, chunks: ChunkReceived, ack: &str)
    -> Result<bool, bool> {
    trace!("Would send received indices for {}, to {} on {}",
           chunks.package, ack, url);
    Ok(true)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use test_library::*;

    use rand;
    use rand::Rng;
    use rustc_serialize::base64;
    use rustc_serialize::base64::ToBase64;

    use handler::HandleMessageParams;
    use message::{BackendServices, PackageId};
    use persistence::Transfer;

    trait Tester<T> { fn new_test(i: usize, package: PackageId) -> T; }

    impl Tester<ChunkParams> for ChunkParams {
        fn new_test(i: usize, package: PackageId) -> ChunkParams {
            let msg = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let b64_msg = msg.as_bytes().to_base64(
                base64::Config {
                    char_set: base64::CharacterSet::UrlSafe,
                    newline: base64::Newline::LF,
                    pad: true,
                    line_length: None
                });

            ChunkParams {
                bytes: b64_msg,
                index: i as u64,
                package: package
            }
        }
    }

    #[test]
    fn it_returns_true_for_existing_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new_test(&prefix);
            let package = transfer.randomize(i);
            let transfers = Mutex::new(HashMap::new());
            transfers.lock().unwrap().insert(package.clone(), transfer);
            let services = Mutex::new(BackendServices::new());

            let chunk = ChunkParams::new_test(i, package);
            assert!(chunk.handle(&services, &transfers, "ignored", "", ""));
        }
    }

    #[test]
    fn it_returns_false_for_nonexisting_transfers() {
        test_init!();
        for i in 1..20 {
            let package = generate_random_package(i);
            let transfers = Mutex::new(HashMap::new());
            let services = Mutex::new(BackendServices::new());

            let chunk = ChunkParams::new_test(i, package);
            assert!(!chunk.handle(&services, &transfers, "ignored", "", ""));
        }
    }
}
