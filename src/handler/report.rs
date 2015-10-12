use std::sync::Mutex;
use std::collections::HashMap;

use message::{BackendServices, PackageId, Notification};
use handler::HandleMessageParams;
use persistence::Transfer;

#[derive(RustcDecodable)]
pub struct ReportParams;

impl HandleMessageParams for ReportParams {
    fn handle(&self,
              _: &Mutex<BackendServices>,
              _: &Mutex<HashMap<PackageId, Transfer>>,
              _: &str, _: &str, _: &str) -> bool {
        true
    }

    fn get_message(&self) -> Option<Notification> {
        Some(Notification::Report)
    }
}
