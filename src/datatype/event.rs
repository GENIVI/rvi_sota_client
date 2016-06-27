use std::string::ToString;

use datatype::{UpdateRequestId, UpdateState, Package};
use datatype::update_request::{UpdateAvailable, DownloadComplete, GetInstalledSoftware};


#[derive(RustcEncodable, RustcDecodable, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Ok,
    Authenticated,
    NotAuthenticated,
    UpdateAvailable(UpdateAvailable),
    DownloadComplete(DownloadComplete),
    GetInstalledSoftware(GetInstalledSoftware),
    UpdateStateChanged(UpdateRequestId, UpdateState),
    UpdateErrored(UpdateRequestId, String),
    Error(String),
    FoundInstalledPackages(Vec<Package>),
}

impl ToString for Event {

    fn to_string(&self) -> String {
        format!("{:?}", *self)
    }

}
