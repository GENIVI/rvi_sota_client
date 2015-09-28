mod service;
mod notify;
mod start;
mod chunk;
mod finish;

use std::sync::Mutex;
use std::collections::HashMap;
use message::{UserMessage, BackendServices, PackageId};
use persistence::Transfer;

pub trait HandleMessageParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<HashMap<PackageId, Transfer>>,
              rvi_url: &str, vin: &str, storage_dir: &str)
        -> bool;

    fn get_message(&self) -> Option<UserMessage>;
}

pub use self::service::ServiceHandler;

pub use self::notify::NotifyParams;
pub use self::start::StartParams;
pub use self::chunk::ChunkParams;
pub use self::finish::FinishParams;
