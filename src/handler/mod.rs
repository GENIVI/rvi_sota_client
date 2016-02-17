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

use std::result;
use std::sync::Mutex;
use message::Notification;
use persistence::Transfers;
pub use self::service::{LocalServices, RemoteServices, ServiceHandler};

pub type Error = bool;
pub type Result =  result::Result<Option<Notification>, Error>;

/// Trait that every message handler needs to implement.
pub trait HandleMessageParams {
    /// Handle the message.
    /// 
    /// Return a [`Notification`](../message/enum.Notification.html) to be passed to the
    /// [`main_loop`](../main_loop/index.html) if apropriate.
    fn handle(&self,
              services: &Mutex<RemoteServices>,
              transfers: &Mutex<Transfers>)
        -> Result;
}

pub use self::notify::NotifyParams;
pub use self::start::StartParams;
pub use self::chunk::ChunkParams;
pub use self::finish::FinishParams;
pub use self::report::ReportParams;
pub use self::abort::AbortParams;
