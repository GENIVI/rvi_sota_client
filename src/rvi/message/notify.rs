use std::sync::Mutex;
use std::collections::HashMap;
use persistence::PackageFile;

use super::HandleMessageParams;
use super::GenericAck;

#[derive(RustcDecodable)]
pub struct NotifyParams {
    pub retry: i32,
    pub package: String
}

impl HandleMessageParams for NotifyParams {
    fn handle(&self,
              pending: &Mutex<HashMap<String, i32>>,
              _: &Mutex<HashMap<u32, PackageFile>>)
        -> bool {

        let mut pending = pending.lock().unwrap();
        pending.insert(self.package.clone(), self.retry);

        info!("New package {} available.", self.package);
        true
    }

    fn get_message(&self) -> String { self.package.clone() }
    fn get_ack(&self) -> Option<GenericAck> { None }
}

