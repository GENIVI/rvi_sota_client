use std::path::Path;

use datatype::Error;
use datatype::Package;


pub trait PackageManager {
    fn installed_packages(&self) -> Result<Vec<Package>, Error>;
    fn install_package(&self, path: &Path) -> Result<(), Error>;
}
