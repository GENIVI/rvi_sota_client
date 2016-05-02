use rustc_serialize::{Decoder, Decodable};

use datatype::{Error, Package, UpdateResultCode};
use package_manager::{dpkg, rpm, tpm};


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PackageManager {
    Dpkg,
    Rpm,
    File { filename: String, succeeds: bool }
}

impl PackageManager {

    pub fn installed_packages(&self) -> Result<Vec<Package>, Error> {
        match *self {
            PackageManager::Dpkg        => dpkg::installed_packages(),
            PackageManager::Rpm         => rpm::installed_packages(),
            PackageManager::File { ref filename, .. } => tpm::installed_packages(filename),
        }
    }

    pub fn install_package(&self, path: &str) -> Result<(UpdateResultCode, String), (UpdateResultCode, String)> {
        match *self {
            PackageManager::Dpkg        => dpkg::install_package(path),
            PackageManager::Rpm         => rpm::install_package(path),
            PackageManager::File { ref filename, succeeds } => tpm::install_package(filename, path, succeeds),
        }
    }

    pub fn extension(&self) -> String {
        match *self {
            PackageManager::Dpkg        => "deb".to_string(),
            PackageManager::Rpm         => "rpm".to_string(),
            PackageManager::File { ref filename, .. } => filename.to_string(),
        }
    }

}

fn parse_package_manager(s: String) -> Result<PackageManager, String> {
    match s.to_lowercase().as_str() {
        "dpkg" => Ok(PackageManager::Dpkg),
        "rpm"  => Ok(PackageManager::Rpm),
        s      => Ok(PackageManager::File { filename: s.to_string(), succeeds: true }),
    }
}

impl Decodable for PackageManager {

    fn decode<D: Decoder>(d: &mut D) -> Result<PackageManager, D::Error> {
        d.read_str().and_then(|s| parse_package_manager(s)
                              .map_err(|e| d.error(&e)))
    }
}
