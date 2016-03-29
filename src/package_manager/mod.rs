pub use self::dpkg::Dpkg;
pub use self::interface::PackageManager;
pub use self::rpm::Rpm;

pub mod dpkg;
pub mod interface;
pub mod rpm;
