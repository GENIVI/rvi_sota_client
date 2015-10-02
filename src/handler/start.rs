use std::sync::Mutex;
use std::collections::HashMap;

use message::{BackendServices, PackageId, ChunkReceived, Notification};
use handler::HandleMessageParams;
use persistence::Transfer;
use rvi::send_message;

#[derive(RustcDecodable)]
pub struct StartParams {
    pub chunkscount: u64,
    pub checksum: String,
    pub package: PackageId,
}

impl HandleMessageParams for StartParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<HashMap<PackageId, Transfer>>,
              rvi_url: &str, vin: &str, storage_dir: &str) -> bool {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();

        info!("Starting transfer for package {}", self.package);

        let transfer =
            Transfer::from_disk(self.package.clone(),
                                self.checksum.clone(),
                                storage_dir.to_string());

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
