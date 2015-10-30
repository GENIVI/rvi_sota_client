//! Handles "Start Transfer" messages.

use std::sync::Mutex;

use message::{BackendServices, PackageId, ChunkReceived, Notification};
use handler::{HandleMessageParams, Transfers};
use persistence::Transfer;
use rvi::send_message;

/// Type for "Start Transfer" messages.
#[derive(RustcDecodable)]
pub struct StartParams {
    /// The amount of chunks this `Transfer` will have.
    pub chunkscount: u64,
    /// The SHA1 checksum of the assembled package.
    pub checksum: String,
    /// The `PackageId` of this `Transfer`.
    pub package: PackageId,
}

impl HandleMessageParams for StartParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              rvi_url: &str, vin: &str, storage_dir: &str) -> bool {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();

        info!("Starting transfer for package {}", self.package);

        let transfer = Transfer::new(storage_dir.to_string(),
                                     self.package.clone(),
                                     self.checksum.clone());

        let chunk_received = ChunkReceived {
            package: self.package.clone(),
            chunks: transfer.transferred_chunks.clone(),
            vin: vin.to_string()
        };

        let _ = transfers.insert(self.package.clone(), transfer);

        try_or!(send_message(rvi_url, chunk_received, &services.ack), return false);
        true
    }

    fn get_message(&self) -> Option<Notification> { None }
}
