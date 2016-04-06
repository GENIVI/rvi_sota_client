use rustc_serialize::{Encodable};
use datatype::UpdateRequestId;

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug)]
pub enum Command {
    // UI
    GetPendingUpdates,
    AcceptUpdate(UpdateRequestId),

    PostInstalledPackages,
    ListInstalledPackages
}
