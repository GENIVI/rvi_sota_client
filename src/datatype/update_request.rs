pub type UpdateRequestId = String;

use datatype::Package;

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct UpdateAvailable {
    pub update_id: String,
    pub signature: String,
    pub description: String,
    pub request_confirmation: bool,
    pub size: u64
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct DownloadComplete {
    pub update_id: String,
    pub update_image: String,
    pub signature: String
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct GetInstalledSoftware {
    pub include_packages: bool,
    pub include_module_firmware: bool
}

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
    pub requestId: UpdateRequestId,
    pub installPos: i32,
    pub packageId: Package,
    pub createdAt: String
}
