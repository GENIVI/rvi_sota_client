use rustc_serialize::{Encodable};

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
