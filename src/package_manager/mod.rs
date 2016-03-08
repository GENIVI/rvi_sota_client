use package::Package;
use error::Error;

pub trait PackageManager: Clone {
    fn installed_packages(&self) -> Result<Vec<Package>, Error>;
}

pub use self::dpkg::Dpkg;

pub mod dpkg;
