use datatype::error::Error;
use datatype::package::Package;


pub trait PackageManager {
    fn new() -> Self;
    fn installed_packages(&self) -> Result<Vec<Package>, Error>;
}
