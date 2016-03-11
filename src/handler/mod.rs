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


use event::inbound::InboundEvent;
use persistence::Transfers;
pub use self::service::{LocalServices, RemoteServices, ServiceHandler};

use std::result;
use std::sync::Mutex;

#[derive(Debug)]
pub enum Error {
    UnknownPackage,
    IoFailure,
    SendFailure
}
pub type Result =  result::Result<Option<InboundEvent>, Error>;

/// Trait that every message handler needs to implement.
pub trait HandleMessageParams {
    /// Handle the message.
    /// 
    /// Return a [`Event`](../message/enum.Event.html) to be passed to the
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
