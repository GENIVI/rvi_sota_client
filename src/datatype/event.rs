use std::fmt;

use datatype::{DownloadComplete, GetInstalledSoftware, Package,
               UpdateAvailable, UpdateRequestId, UpdateState};


#[derive(RustcEncodable, RustcDecodable, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Ok,
    Error(String),

    Authenticated,
    NotAuthenticated,

    GetInstalledSoftware(GetInstalledSoftware),
    FoundInstalledPackages(Vec<Package>),

    UpdateAvailable(UpdateAvailable),
    UpdateStateChanged(UpdateRequestId, UpdateState),
    DownloadComplete(DownloadComplete),
    UpdateErrored(UpdateRequestId, String),
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
