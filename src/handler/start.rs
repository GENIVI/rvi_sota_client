//! Handles "Start Transfer" messages.

use std::sync::Mutex;

use event::UpdateId;
use message::ChunkReceived;
use handler::{HandleMessageParams, RemoteServices, Result, Error};
use persistence::Transfers;

/// Type for "Start Transfer" messages.
#[derive(RustcDecodable)]
pub struct StartParams {
    pub update_id: UpdateId,
    /// The amount of chunks this `Transfer` will have.
    pub chunkscount: u64,
    /// The SHA1 checksum of the assembled package.
    pub checksum: String
}

impl HandleMessageParams for StartParams {
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>) -> Result {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();

        info!("Starting transfer for update_id {}", self.update_id);

        transfers.push(self.update_id.clone(), self.checksum.clone());
        services.send_chunk_received(
            ChunkReceived {
                update_id: self.update_id.clone(),
                chunks: Vec::new(),
                vin: services.vin.clone() })
            .map_err(|e| {
                error!("Error on sending start ACK: {}", e);
                Error::SendFailure })
            .map(|_| None)
    }
}
