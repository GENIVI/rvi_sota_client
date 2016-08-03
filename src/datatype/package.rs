use std::fmt::{Display, Formatter, Result as FmtResult};

use rvi::services::LocalServices;


pub type Version = String;

#[derive(Debug, PartialEq, Eq, RustcEncodable, RustcDecodable, Clone)]
pub struct Package {
    pub name:    String,
    pub version: Version
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", self.name, self.version)
    }
}


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum UpdateState {
    Downloading,
    Installing,
    Installed,
    Failed,
}


pub type UpdateRequestId = String;

#[allow(non_snake_case)]
#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct PendingUpdateRequest {
    pub requestId:  UpdateRequestId,
    pub installPos: i32,
    pub packageId:  Package,
    pub createdAt:  String
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct UpdateAvailable {
    pub update_id:    String,
    pub signature:    String,
    pub description:  String,
    pub confirmation: bool,
    pub size:         u64
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct StartDownload {
    pub update_id: UpdateRequestId,
    pub device_id: String,
    pub local:     LocalServices,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct ChunkReceived {
    pub update_id: UpdateRequestId,
    pub device_id: String,
    pub chunks:    Vec<u64>,
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct DownloadComplete {
    pub update_id:    String,
    pub update_image: String,
    pub signature:    String
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct GetInstalledSoftware {
    pub include_packages: bool,
    pub include_firmware: bool
}
