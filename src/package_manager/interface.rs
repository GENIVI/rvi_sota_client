use datatype::Error;
use datatype::Package;


pub trait PackageManager {
    fn new() -> Self;
    fn installed_packages(&self) -> Result<Vec<Package>, Error>;
}
