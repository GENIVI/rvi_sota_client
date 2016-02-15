//! Implements message handling for the `rvi` module.
//!
//! Implements individual message handler for the different messages the client might receive, and
//! the general `ServiceHandler` that parses messages and delegates them to the individual
//! handlers.
//!
//! This is a reference implementation for the [`rvi`](../rvi/index.html) module, that can later be
//! split out into a crate.

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

/// Type alias to hide the internal `HashMap`, that is used to store
/// [`Transfer`](../persistence/struct.Transfer.html)s.
pub type Transfers = HashMap<PackageId, Transfer>;

/// Trait that every message handler needs to implement.
pub trait HandleMessageParams {
    /// Handle the message. Returns a `bool` to indicate success or failure.
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              rvi_url: &str, vin: &str, storage_dir: &str)
        -> bool;

    /// Return a [`Notification`](../message/enum.Notification.html) to be passed to the
    /// [`main_loop`](../main_loop/index.html) if apropriate.
    fn get_message(&self) -> Option<Notification>;
}

pub use self::service::LocalServices;
pub use self::service::ServiceHandler;

pub use self::notify::NotifyParams;
pub use self::start::StartParams;
pub use self::chunk::ChunkParams;
pub use self::finish::FinishParams;
pub use self::report::ReportParams;
pub use self::abort::AbortParams;
