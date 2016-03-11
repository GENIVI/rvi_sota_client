//! Handles messages transferring single chunks.

use std::sync::Mutex;

use event::UpdateId;
use message::ChunkReceived;
use handler::{Error, Result, RemoteServices, HandleMessageParams};
use persistence::Transfers;

/// Type for messages transferring single chunks.
#[derive(RustcDecodable)]
pub struct ChunkParams {
    /// The package transfer this chunk belongs to.
    pub update_id: UpdateId,
    /// The data of the transferred chunk.
    pub bytes: String,
    /// The index of this chunk.
    pub index: u64
}

impl HandleMessageParams for ChunkParams {
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>) -> Result {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();
        transfers.get_mut(&self.update_id).map(|t| {
            if t.write_chunk(&self.bytes, self.index) {
                info!("Wrote chunk {} for package {}", self.index, self.update_id);
                services.send_chunk_received(
                    ChunkReceived {
                        update_id: self.update_id.clone(),
                        chunks: t.transferred_chunks.clone(),
                        vin: services.vin.clone() })
                    .map_err(|e| {
                        error!("Error on sending ChunkReceived: {}", e);
                        Error::SendFailure })
                    .map(|_| None)
            } else {
                Err(Error::IoFailure)
            }
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for update_id {}", self.update_id);
            Err(Error::UnknownPackage)
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use super::*;
    use test_library::*;

    use rand;
    use rand::Rng;
    use rustc_serialize::base64;
    use rustc_serialize::base64::ToBase64;

    use handler::HandleMessageParams;
    use message::PackageId;
    use persistence::{Transfer, Transfers};

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
            let transfers = Mutex::new(Transfers::new(prefix.to_string()));
            transfers.lock().unwrap().push_test(transfer);
            let services = Mutex::new(get_empty_backend());

            let chunk = ChunkParams::new_test(i, package);
            assert!(chunk.handle(&services, &transfers).is_ok());
        }
    }

    #[test]
    fn it_returns_false_for_nonexisting_transfers() {
        test_init!();
        for i in 1..20 {
            let package = generate_random_package(i);
            let transfers = Mutex::new(Transfers::new("".to_string()));
            let services = Mutex::new(get_empty_backend());

            let chunk = ChunkParams::new_test(i, package);
            assert!(!chunk.handle(&services, &transfers).is_ok());
        }
    }
}
