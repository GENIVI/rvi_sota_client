use std::sync::Mutex;
use std::collections::HashMap;
use persistence::PackageFile;

use std::vec;

// Declare submodules
mod notify;
mod start;
mod chunk;
mod finish;
mod params;

// Reexport all message types
pub use self::notify::NotifyParams;
pub use self::start::StartParams;
pub use self::chunk::ChunkParams;
pub use self::finish::FinishParams;
pub use self::params::*;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Message<T> {
    pub service_name: String,
    pub parameters: vec::Vec<T>
}

#[derive(RustcDecodable)]
pub enum MessageEventParams {
    Notify(NotifyParams),
    Start(StartParams),
    Chunk(ChunkParams),
    Finish(FinishParams),
    Ack(AckParams)
}

pub trait HandleMessageParams {
    fn handle(&self,
              pending: &Mutex<HashMap<String, i32>>,
              transfers: &Mutex<HashMap<u32, PackageFile>>)
        -> bool;
    fn get_message(&self) -> String;
    fn get_ack(&self) -> Option<GenericAck>;
}
