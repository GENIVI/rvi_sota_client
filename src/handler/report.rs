//! Handles "Get All Packages" messages.

use std::sync::Mutex;

use message::{BackendServices, Notification};
use handler::{Result, HandleMessageParams};
use persistence::Transfers;

#[derive(RustcDecodable)]
/// Type for "Get All Packages" messages.
pub struct ReportParams;

impl HandleMessageParams for ReportParams {
    fn handle(&self,
              _: &Mutex<BackendServices>,
              _: &Mutex<Transfers>,
              _: &str,
              _: &str) -> Result {
        Ok(Some(Notification::Report))
    }
}
