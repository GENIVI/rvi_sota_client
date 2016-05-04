use std::string::ToString;

use datatype::{UpdateRequestId, UpdateState, Package};


#[derive(RustcEncodable, RustcDecodable, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Ok,
    NotAuthenticated,
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
