use datatype::{Error, OtaConfig, Package, UpdateResultCode};

pub trait PackageManager {
    fn installed_packages(&self, &OtaConfig) -> Result<Vec<Package>, Error>;
    fn install_package(&self, &OtaConfig, path: &str) -> Result<(UpdateResultCode, String), (UpdateResultCode, String)>;
}
