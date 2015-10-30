//! Helper functions for the "Initiate Download" message, that gets sent to the server.

use super::package_id::PackageId;
use rvi::Service;

/// Encodes the list of service URLs the client registered.
///
/// Needs to be extended to introduce new services.
#[derive(RustcEncodable, Clone)]
pub struct LocalServices {
    /// "Start Download" URL.
    pub start: String,
    /// "Chunk" URL.
    pub chunk: String,
    /// "Abort Download" URL.
    pub abort: String,
    /// "Finish Download" URL.
    pub finish: String,
    /// "Get All Packages" URL.
    pub getpackages: String,
}

impl LocalServices {
    /// Parses the given `Vector` of service URLs into a `LocalServices` object.
    ///
    /// # Arguments
    /// * `s`: `Vector` with the URLs returned from RVI at registration time.
    pub fn new(s: &Vec<Service>) -> LocalServices {
        let mut services = LocalServices {
            start: "".to_string(),
            chunk: "".to_string(),
            abort: "".to_string(),
            finish: "".to_string(),
            getpackages: "".to_string()
        };

        for service in s {
            let serv = &mut services;
            match service.name.as_ref() {
                "/sota/start" => serv.start = service.addr.clone(),
                "/sota/chunk" => serv.chunk = service.addr.clone(),
                "/sota/abort" => serv.abort = service.addr.clone(),
                "/sota/finish" => serv.finish = service.addr.clone(),
                "/sota/getpackages" => serv.getpackages = service.addr.clone(),
                _ => {}
            }
        }

        services
    }

    /// Returns the VIN of this device.
    ///
    /// # Arguments
    /// * `vin_match`: The index, where to look for the VIN in the service URL.
    pub fn get_vin(&self, vin_match: i32) -> String {
        self.start.split("/").nth(vin_match as usize).unwrap().to_string()
    }
}

/// Encodes the parameters needed for the "Initiate Download" message.
#[derive(RustcEncodable)]
pub struct InitiateParams {
    /// `Vector` of packages that should be updated.
    pub packages: Vec<PackageId>,
    /// `LocalServices` object with the service URLs of this device.
    pub services: LocalServices,
    /// The VIN of this device.
    pub vin: String
}

impl InitiateParams {
    /// Creates a new `InitateParams` object.
    ///
    /// # Arguments
    /// * `p`: The package to update. Is wrapped into a `Vector` as a translation layer between
    ///   SOTA server and the Software Loading Manager.
    /// * `s`: The `LocalServices` this device supports.
    /// * `v`: The VIN of this device.
    pub fn new(p: PackageId, s: LocalServices,
               v: String) -> InitiateParams {
        InitiateParams {
            packages: vec!(p),
            services: s,
            vin: v
        }
    }
}
