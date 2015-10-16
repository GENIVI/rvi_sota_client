use super::package_id::PackageId;
use super::client::PackageReport;

#[derive(RustcEncodable)]
pub struct ChunkReceived {
    pub package: PackageId,
    pub chunks: Vec<u64>,
    pub vin: String
}

#[derive(RustcDecodable, Clone)]
pub struct BackendServices {
    pub start: String,
    pub cancel: String,
    pub ack: String,
    pub report: String,
    pub packages: String
}

impl BackendServices {
    pub fn new() -> BackendServices {
        BackendServices {
            start: "".to_string(),
            cancel: "".to_string(),
            ack: "".to_string(),
            report: "".to_string(),
            packages: "".to_string()
        }
    }

    pub fn update(&mut self, new: &BackendServices) {
        self.start = new.start.clone();
        self.cancel = new.cancel.clone();
        self.ack = new.ack.clone();
        self.report = new.report.clone();
        self.packages = new.packages.clone();
    }
}

#[derive(RustcDecodable, Clone)]
pub struct PackageSum {
    pub package: PackageId,
    pub checksum: String
}

#[derive(RustcEncodable)]
pub struct ServerPackageReport {
    pub package: PackageId,
    pub status: bool,
    pub description: String,
    pub vin: String
}

impl ServerPackageReport {
    pub fn new(r: PackageReport, v: String) -> ServerPackageReport {
        ServerPackageReport {
            package: r.package,
            status: r.status,
            description: r.description,
            vin: v
        }
    }
}

#[derive(RustcEncodable)]
pub struct ServerReport {
    pub packages: Vec<PackageId>,
    pub vin: String
}

impl ServerReport {
    pub fn new(p: Vec<PackageId>, v: String) -> ServerReport {
        ServerReport {
            packages: p,
            vin: v
        }
    }
}
