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
    pub update_id:            String,
    pub signature:            String,
    pub description:          String,
    pub request_confirmation: bool,
    pub size:                 u64
}

/// A JSON-RPC request type to notify RVI that a new package download has started.
#[derive(RustcDecodable, RustcEncodable)]
pub struct DownloadStarted {
    pub device:    String,
    pub update_id: UpdateRequestId,
    pub services:  LocalServices,
}

/// A JSON-RPC request type to notify RVI that a new package chunk was received.
#[derive(RustcDecodable, RustcEncodable)]
pub struct ChunkReceived {
    pub device:    String,
    pub update_id: UpdateRequestId,
    pub chunks:    Vec<u64>,
}

/// A notification to indicate to any external package manager that the package
/// download has successfully completed.
#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub struct DownloadComplete {
    pub update_id:    String,
    pub update_image: String,
    pub signature:    String
}

impl Default for DownloadComplete {
    fn default() -> Self {
        DownloadComplete {
            update_id:    "".to_string(),
            update_image: "".to_string(),
            signature:    "".to_string()
        }
    }
}
