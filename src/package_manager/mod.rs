extern crate tempfile;

pub use self::package_manager::PackageManager;
pub use self::tpm::assert_rx;

pub mod dpkg;
pub mod package_manager;
pub mod rpm;
pub mod tpm;
