//! Handles "Get All Packages" messages.

use std::sync::Mutex;

use message::{BackendServices, Notification};
use handler::{Transfers, HandleMessageParams};

#[derive(RustcDecodable)]
/// Type for "Get All Packages" messages.
pub struct ReportParams;

impl HandleMessageParams for ReportParams {
    fn handle(&self,
              _: &Mutex<BackendServices>,
              _: &Mutex<Transfers>,
              _: &str, _: &str, _: &str) -> bool {
        true
    }

    fn get_message(&self) -> Option<Notification> {
        Some(Notification::Report)
    }
}
