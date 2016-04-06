use rustc_serialize::{Encodable};

use datatype::{UpdateRequestId, UpdateState};

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum Event {
    NewUpdateAvailable(UpdateRequestId),
    UpdateStateChanged(UpdateRequestId, UpdateState),
    UpdateErrored(UpdateRequestId, String),
    Error(String),
    Batch(Vec<Event>)
}
