use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use persistence::PackageFile;

use std::vec;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Message<T> {
    pub service_name: String,
    pub parameters: vec::Vec<T>
}

#[derive(RustcDecodable)]
pub struct NotifyParams {
    pub retry: i32,
    pub package: String
}

#[derive(RustcDecodable)]
pub struct StartParams {
    pub id: u32,
    pub total_size: i32,
    pub package: String,
    pub chunk_size: i32
}

#[derive(RustcDecodable)]
pub struct ChunkParams {
    pub id: u32,
    pub index: i32,
    pub msg: String
}

#[derive(RustcDecodable)]
pub struct FinishParams {
    pub id: u32
}

#[derive(RustcEncodable)]
pub struct InitiateParams {
    pub id: u32,
    pub package: String,
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct AckParams {
    id: u32,
    ack: String
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct AckChunkParams {
    id: u32,
    ack: String,
    index: i32
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum GenericAck {
    Ack(AckParams),
    Chunk(AckChunkParams)
}

#[derive(RustcDecodable)]
pub enum MessageEventParams {
    Notify(NotifyParams),
    Start(StartParams),
    Chunk(ChunkParams),
    Finish(FinishParams),
    Ack(AckParams)
}

// #[derive(RustcEncodable)]
// pub enum MessageSendParams {
//     Init(InitiateParams),
//     Ack(AckParams)
// }

pub trait HandleMessageParams {
    fn handle(&self,
              pending: &Mutex<HashMap<String, i32>>,
              transfers: &Mutex<HashMap<u32, PackageFile>>)
        -> bool;
    fn get_message(&self) -> String;
    fn get_ack(&self) -> Option<GenericAck>;
}

impl HandleMessageParams for NotifyParams {
    fn handle(&self,
              pending: &Mutex<HashMap<String, i32>>,
              _: &Mutex<HashMap<u32, PackageFile>>)
        -> bool {

        let mut pending = pending.lock().unwrap();
        pending.insert(self.package.clone(), self.retry);
        true
    }

    fn get_message(&self) -> String { self.package.clone() }
    fn get_ack(&self) -> Option<GenericAck> { None }
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
                println!("!!! ERR unknown package {} with id {} in start", self.package, self.id);
                return false;
            }
        }

        let pfile = PackageFile::new(&self.package,
                                     self.total_size,
                                     self.chunk_size);

        let mut i = 1;
        while transfers.contains_key(&i) {
            i = i + 1;
        }

        transfers.insert(i, pfile);
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
                println!("!!! ERR unknown id {} in chunk", self.id);
                return false;
            }
        }

        if is_finished {
            transfers.remove(&self.id);
        }

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
                println!("!!! ERR unknown id {} in finish", self.id);
                return false;
            }
        }

        if is_finished {
            transfers.remove(&self.id);
        }

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
