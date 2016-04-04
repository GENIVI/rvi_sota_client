use datatype::Error;
use datatype::OtaConfig;
use datatype::Package;


pub trait PackageManager {
    fn installed_packages(&self, &OtaConfig) -> Result<Vec<Package>, Error>;
    fn install_package(&self, &OtaConfig, path: &str) -> Result<(), Error>;
}
