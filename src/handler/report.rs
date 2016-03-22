//! Handles "Get All Packages" messages.

use std::sync::Mutex;

use event::inbound::{InboundEvent, GetInstalledSoftware};
use handler::{Result, RemoteServices, HandleMessageParams};
use persistence::Transfers;

/// Type for "Get All Packages" messages.
pub type ReportParams = GetInstalledSoftware;

impl HandleMessageParams for ReportParams {
    fn handle(&self,
              _: &Mutex<RemoteServices>,
              _: &Mutex<Transfers>) -> Result {
        Ok(Some(InboundEvent::GetInstalledSoftware(self.clone())))
    }
}
