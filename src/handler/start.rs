//! Handles "Start Transfer" messages.

use std::sync::Mutex;

use message::{BackendServices, PackageId, ChunkReceived};
use handler::{HandleMessageParams, Result};
use persistence::Transfers;
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
              rvi_url: &str,
              vin: &str) -> Result {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();

        info!("Starting transfer for package {}", self.package);

        transfers.push(self.package.clone(), self.checksum.clone());
        let chunk_received = ChunkReceived {
            package: self.package.clone(),
            chunks: Vec::new(),
            vin: vin.to_string()
        };
        send_message(rvi_url, chunk_received, &services.ack)
            .map_err(|e| {error!("Error on sending start ACK: {}", e); false})
            .map(|_| None)
    }
}
