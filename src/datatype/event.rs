use std::fmt::{Display, Formatter, Result as FmtResult};

use datatype::{DownloadComplete, Package, UpdateAvailable, UpdateReport,
               UpdateRequest, UpdateRequestId};


/// System-wide events that are broadcast to all interested parties.
#[derive(RustcEncodable, RustcDecodable, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// General error event with a printable representation for debugging.
    Error(String),

    /// Authentication was successful.
    Authenticated,
    /// An operation failed because we are not currently authenticated.
    NotAuthenticated,
    /// Nothing was done as we are already authenticated.
    AlreadyAuthenticated,

    /// A notification from Core of pending or in-flight updates.
    UpdatesReceived(Vec<UpdateRequest>),
    /// A notification from RVI of a pending update.
    UpdateAvailable(UpdateAvailable),
    /// There are no outstanding update requests.
    NoUpdateRequests,

    /// The following packages are installed on the device.
    FoundInstalledPackages(Vec<Package>),
    /// An update on the system information was received.
    FoundSystemInfo(String),

    /// Downloading an update.
    DownloadingUpdate(UpdateRequestId),
    /// An update was downloaded.
    DownloadComplete(DownloadComplete),
    /// Downloading an update failed.
    DownloadFailed(UpdateRequestId, String),

    /// Installing an update.
    InstallingUpdate(UpdateRequestId),
    /// An update was installed.
    InstallComplete(UpdateReport),
    /// The installation of an update failed.
    InstallFailed(UpdateReport),

    /// An update report was sent to the Core server.
    UpdateReportSent,
    /// A list of installed packages was sent to the Core server.
    InstalledPackagesSent,
    /// A list of installed software was sent to the Core server.
    InstalledSoftwareSent,
    /// The system information was sent to the Core server.
    SystemInfoSent,

    /// A broadcast event requesting an update on externally installed software.
    InstalledSoftwareNeeded,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
