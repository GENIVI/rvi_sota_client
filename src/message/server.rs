//! Translation layer for the SOTA server.

use event::UpdateId;

/// Encodes the "Chunk Received" message, indicating that a chunk was successfully transferred.
#[derive(RustcEncodable)]
pub struct ChunkReceived {
    /// The transfer to which the transferred chunk belongs.
    pub update_id: UpdateId,
    /// A list of the successfully transferred chunks.
    pub chunks: Vec<u64>,
    /// The VIN of this device.
    pub vin: String
}

/// Encodes the service URLs, that the server provides.
#[derive(RustcDecodable, Clone)]
pub struct BackendServices {
    /// URL for the "Start Download" call.
    pub start: String,
    /// URL for the "Chunk Received" call.
    pub ack: String,
    /// URL for the "Installation Report" call.
    pub report: String,
    /// URL for the "Get All Packages" call.
    pub packages: String
}

impl BackendServices {
    /// Creates a new, empty `BackendServices` object.
    pub fn new() -> BackendServices {
        BackendServices {
            start: "".to_string(),
            ack: "".to_string(),
            report: "".to_string(),
            packages: "".to_string()
        }
    }

    /*
    /// Update the `BackendServices` object with new values.
    ///
    /// # Arguments
    /// * `new`: A pointer to another `BackendServices` object, that should be cloned.
    pub fn update(&mut self, new: &BackendServices) {
        self.start = new.start.clone();
        self.ack = new.ack.clone();
        self.report = new.report.clone();
        self.packages = new.packages.clone();
    }
    */
}

/*
use super::package_id::PackageId;

/// Encodes a package/checksum pair.
#[derive(RustcDecodable, Clone)]
pub struct PackageSum {
    /// The package for which the checksum is valid.
    pub package: PackageId,
    /// The checksum of the package.
    pub checksum: String
}

/// Encodes a package installation report, as required by the SOTA server.
#[derive(RustcEncodable)]
pub struct ServerPackageReport {
    /// The package that got installed.
    pub package: PackageId,
    /// Boolean to indicate success or failure.
    pub status: bool,
    /// A short description of the result of the installation request.
    pub description: String,
    /// The VIN of this device.
    pub vin: String
}

use super::client::PackageReport;
impl ServerPackageReport {
    /// Create a new `ServerPackageReport` from a `PackageReport`.
    ///
    /// # Arguments
    /// * `r`: The `PackageReport` to translate for the server.
    /// * `v`: The VIN of this device.
    pub fn new(r: PackageReport, v: String) -> ServerPackageReport {
        ServerPackageReport {
            package: r.package,
            status: r.status,
            description: r.description,
            vin: v
        }
    }
}

/// Encodes a installed packages report, as required by the SOTA server.
#[derive(RustcEncodable)]
pub struct ServerReport {
    /// A list of packages, that are installed on this device.
    pub packages: Vec<PackageId>,
    /// The vin of this device.
    pub vin: String
}

impl ServerReport {
    /// Create a new `ServerReport` from a list of `PackageId`s.
    ///
    /// # Arguments
    /// * `p`: A list of packages that are installed on this device.
    /// * `v`: The VIN of this device.
    pub fn new(p: Vec<PackageId>, v: String) -> ServerReport {
        ServerReport {
            packages: p,
            vin: v
        }
    }
}
*/
