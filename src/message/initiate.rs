//! Helper functions for the "Initiate Download" message, that gets sent to the server.

/*
use handler::LocalServices;
use super::package_id::PackageId;
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
*/
