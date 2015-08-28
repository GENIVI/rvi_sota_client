use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use persistence::PackageFile;

use super::HandleMessageParams;
use super::GenericAck;
use super::AckParams;

#[derive(RustcDecodable)]
pub struct StartParams {
    pub id: u32,
    pub total_size: i32,
    pub package: String,
    pub chunk_size: i32
}

impl HandleMessageParams for StartParams {
    fn handle(&self,
              pending: &Mutex<HashMap<String, i32>>,
              transfers: &Mutex<HashMap<u32, PackageFile>>)
        -> bool {

        let mut transfers = transfers.lock().unwrap();
        let mut pending = pending.lock().unwrap();

        // make sure this transfer was announced
        // TODO: update retry count, fail accordingly
        match pending.entry(self.package.clone()) {
            Entry::Occupied(_) => {},
            Entry::Vacant(..) => {
                error!("!!! ERR: unknown package {} with id {} in start", self.package, self.id);
                return false;
            }
        }

        let pfile = PackageFile::new(&self.package,
                                     self.total_size,
                                     self.chunk_size);
        transfers.insert(self.id, pfile);

        info!("Started transfer #{} for package {}.", self.id, self.package);
        true
    }

    fn get_message(&self) -> String { self.id.to_string() }
    fn get_ack(&self) -> Option<GenericAck> {
        Some(GenericAck::Ack(AckParams {
            id: self.id,
            ack: "start".to_string(),
        }))
    }
}
