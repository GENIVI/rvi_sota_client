pub type UpdateRequestId = String;

use datatype::Package;

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum UpdateState {
    Downloading,
    Installing,
    Installed,
    Failed,
}

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct PendingUpdateRequest {
    pub id:     UpdateRequestId,
    pub packageId: Package,
    pub createdAt: String
}
