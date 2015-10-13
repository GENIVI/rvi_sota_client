mod service;
mod notify;
mod start;
mod chunk;
mod finish;
mod report;
mod abort;

use std::sync::Mutex;
use std::collections::HashMap;
use message::{BackendServices, PackageId, Notification};
use persistence::Transfer;

pub type Transfers = HashMap<PackageId, Transfer>;
pub trait HandleMessageParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              rvi_url: &str, vin: &str, storage_dir: &str)
        -> bool;

    fn get_message(&self) -> Option<Notification>;
}

pub use self::service::ServiceHandler;

pub use self::notify::NotifyParams;
pub use self::start::StartParams;
pub use self::chunk::ChunkParams;
pub use self::finish::FinishParams;
pub use self::report::ReportParams;
pub use self::abort::AbortParams;
