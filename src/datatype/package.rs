use std::fmt::{Display, Formatter, Result as FmtResult};

use rvi::services::LocalServices;


/// Encapsulate a `String` type used to represent the `Package` version.
pub type Version = String;

/// Encodes the name and version of a specific package.
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


/// Track the transition states when installing a new package.
#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum UpdateState {
    Downloading,
    Installing,
    Installed,
    Failed,
}


/// Encapsulate a `String` type as the id of a specific update request.
pub type UpdateRequestId = String;

/// A single pending update request to be installed by the client.
#[allow(non_snake_case)]
#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct PendingUpdateRequest {
    pub requestId:  UpdateRequestId,
    pub installPos: i32,
    pub packageId:  Package,
    pub createdAt:  String
}

/// A notification from RVI that a new update is available.
#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct UpdateAvailable {
    pub update_id:    String,
    pub signature:    String,
    pub description:  String,
    pub confirmation: bool,
    pub size:         u64
}

/// A JSON-RPC request type to notify RVI that a new package download has started.
#[derive(RustcDecodable, RustcEncodable)]
pub struct DownloadStarted {
    pub update_id: UpdateRequestId,
    pub device_id: String,
    pub local:     LocalServices,
}

/// A JSON-RPC request type to notify RVI that a new package chunk was received.
#[derive(RustcDecodable, RustcEncodable)]
pub struct ChunkReceived {
    pub update_id: UpdateRequestId,
    pub device_id: String,
    pub chunks:    Vec<u64>,
}

/// A notification from RVI to indicate the package download is complete.
#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct DownloadComplete {
    pub update_id:    String,
    pub update_image: String,
    pub signature:    String
}

/// A notification from RVI requesting a report on the installed software.
#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct GetInstalledSoftware {
    pub include_packages: bool,
    pub include_firmware: bool
}
