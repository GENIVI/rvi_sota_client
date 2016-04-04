use datatype::Error;
use datatype::OtaConfig;
use datatype::Package;
use package_manager::PackageManager;


pub struct Rpm;

pub static RPM: &'static PackageManager = &Rpm;

impl PackageManager for Rpm {

    fn installed_packages(&self, _: &OtaConfig) -> Result<Vec<Package>, Error> {
        unimplemented!();
    }

    fn install_package(&self, _: &OtaConfig, _: &str) -> Result<(), Error> {
        unimplemented!();
    }

}
