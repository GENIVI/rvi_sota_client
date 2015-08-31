use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use persistence::PackageFile;

use super::HandleMessageParams;
use super::GenericAck;
use super::AckParams;

#[derive(RustcDecodable)]
pub struct FinishParams {
    pub id: u32
}

impl HandleMessageParams for FinishParams {
    fn handle(&self,
              pending: &Mutex<HashMap<String, i32>>,
              transfers: &Mutex<HashMap<u32, PackageFile>>)
        -> bool {

        let mut pending = pending.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();
        let mut is_finished: bool;

        match transfers.entry(self.id) {
            Entry::Occupied(mut entry) => {
                let mut package = entry.get_mut();
                is_finished = package.finish();
                pending.remove(&package.package_name);
            },
            Entry::Vacant(..) => {
                error!("!!! ERR: unknown id {} in finish", self.id);
                return false;
            }
        }

        if is_finished {
            transfers.remove(&self.id);
        }

        info!("Finished transfer #{}.", self.id);
        true
    }

    fn get_message(&self) -> String {
        return self.id.to_string();
    }

    fn get_ack(&self) -> Option<GenericAck> {
        Some(GenericAck::Ack(AckParams {
            id: self.id,
            ack: "finish".to_string(),
        }))
    }
}
