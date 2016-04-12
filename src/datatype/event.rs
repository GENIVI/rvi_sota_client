use rustc_serialize::{Encodable};
use std::string::ToString;

use datatype::{UpdateRequestId, UpdateState, Package};


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum Event {
    NewUpdateAvailable(UpdateRequestId),
    UpdateStateChanged(UpdateRequestId, UpdateState),
    UpdateErrored(UpdateRequestId, String),
    Error(String),
    FoundInstalledPackages(Vec<Package>),
    Batch(Vec<Event>)
}

impl ToString for Event {

    fn to_string(&self) -> String {
        format!("{:?}", *self)
    }

}
