use super::package_id::PackageId;

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
    pub report: String
}

impl BackendServices {
    pub fn new() -> BackendServices {
        BackendServices {
            start: "".to_string(),
            cancel: "".to_string(),
            ack: "".to_string(),
            report: "".to_string()
        }
    }

    pub fn update(&mut self, new: &BackendServices) {
        self.start = new.start.clone();
        self.cancel = new.cancel.clone();
        self.ack = new.ack.clone();
        self.report = new.report.clone();
    }
}

#[derive(RustcDecodable, Clone)]
pub struct PackageSum {
    pub package: PackageId,
    pub checksum: String
}
