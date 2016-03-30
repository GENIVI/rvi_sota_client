pub use self::dpkg::Dpkg;
pub use self::interface::PackageManager;
pub use self::rpm::Rpm;
pub use self::tpm::Tpm;

pub mod dpkg;
pub mod interface;
pub mod rpm;
pub mod tpm;
