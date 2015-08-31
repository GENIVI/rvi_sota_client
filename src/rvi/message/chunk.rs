use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use persistence::PackageFile;

use super::HandleMessageParams;
use super::GenericAck;
use super::AckChunkParams;

#[derive(RustcDecodable)]
pub struct ChunkParams {
    pub id: u32,
    pub index: i32,
    pub msg: String
}

impl HandleMessageParams for ChunkParams {
    fn handle(&self,
              pending: &Mutex<HashMap<String, i32>>,
              transfers: &Mutex<HashMap<u32, PackageFile>>)
        -> bool {

        let mut transfers = transfers.lock().unwrap();
        let mut pending = pending.lock().unwrap();
        let mut is_finished = false;

        match transfers.entry(self.id) {
            Entry::Occupied(mut entry) => {
                let mut package = entry.get_mut();
                package.write_chunk(&(self.msg), self.index);
                if package.is_finished() {
                    is_finished = package.finish();
                    pending.remove(&package.package_name);
                }
            },
            Entry::Vacant(..) => {
                error!("!!! ERR: unknown id {} in chunk", self.id);
                return false;
            }
        }

        if is_finished {
            transfers.remove(&self.id);
        }

        info!("Got chunk #{} for transfer #{}.", self.index, self.id);
        true
    }

    fn get_message(&self) -> String { return self.id.to_string(); }
    fn get_ack(&self) -> Option<GenericAck> {
        Some(GenericAck::Chunk(AckChunkParams {
            id: self.id,
            ack: "chunk".to_string(),
            index: self.index
        }))
    }
}
