use rustc_serialize::{Encodable};

use datatype::{UpdateRequestId, UpdateState, Package};
use interaction_library::Print;


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum Event {
    NewUpdateAvailable(UpdateRequestId),
    UpdateStateChanged(UpdateRequestId, UpdateState),
    UpdateErrored(UpdateRequestId, String),
    Error(String),
    FoundInstalledPackages(Vec<Package>),
    Batch(Vec<Event>)
}

impl Print for Event {

    fn pretty_print(&self) -> String {
        format!("{:?}", *self)
    }

}
