use std::fmt::{Display, Formatter, Result as FmtResult};

use datatype::{DownloadComplete, GetInstalledSoftware, Package,
               UpdateAvailable, UpdateRequestId, UpdateState};


/// System-wide events that are broadcast to all interested parties.
#[derive(RustcEncodable, RustcDecodable, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// No-op event to signal the success case.
    Ok,
    /// General error event with a printable representation for debugging.
    Error(String),

    /// Notification the authentication was successful.
    Authenticated,
    /// An operation failed because we are not currently authenticated.
    NotAuthenticated,

    /// An event used to notify the DBus gateway to call the respective method.
    GetInstalledSoftware(GetInstalledSoftware),
    /// A list of the currently installed system packages.
    FoundInstalledPackages(Vec<Package>),

    /// A new update has been found.
    UpdateAvailable(UpdateAvailable),
    /// The installation of a specific update has progressed to a new state.
    UpdateStateChanged(UpdateRequestId, UpdateState),
    /// Downloading a specific update has successfully completed.
    DownloadComplete(DownloadComplete),
    /// The installation of a specific update failed.
    UpdateErrored(UpdateRequestId, String),

    GotSystemInfo(String),
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
