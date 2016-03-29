use std::path::Path;

use datatype::Error;
use datatype::Package;
use package_manager::PackageManager;


pub struct Rpm;

impl PackageManager for Rpm {

    fn new() -> Rpm {
        return Rpm
    }

    fn installed_packages(&self) -> Result<Vec<Package>, Error> {
        unimplemented!();
    }

    fn install_package(&self, _: &Path) -> Result<(), Error> {
        unimplemented!();
    }

}
