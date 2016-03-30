use datatype::Error;
use datatype::Package;
use package_manager::PackageManager;


pub struct Rpm;

pub static RPM: &'static PackageManager = &Rpm;

impl PackageManager for Rpm {

    fn installed_packages(&self) -> Result<Vec<Package>, Error> {
        unimplemented!();
    }

    fn install_package(&self, _: &str) -> Result<(), Error> {
        unimplemented!();
    }

}
