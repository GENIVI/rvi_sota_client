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
#[allow(non_snake_case)]
pub struct PendingUpdateRequest {
    pub id:     UpdateRequestId,
    pub installPos: i32,
    pub packageId: Package,
    pub createdAt: String
}
