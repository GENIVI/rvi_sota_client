use rustc_serialize::{Decoder, Decodable};


#[derive(Debug, PartialEq, Eq)]
pub enum PackageManager {
    Dpkg,
    Rpm,
    Test,
}

impl PackageManager {

    pub fn extension(&self) -> String {
        match *self {
            PackageManager::Dpkg => "deb".to_string(),
            PackageManager::Rpm  => "rpm".to_string(),
            PackageManager::Test => "test".to_string(),
        }
    }

}

fn parse_package_manager(s: String) -> Result<PackageManager, String> {
    match s.to_lowercase().as_str() {
        "dpkg" => Ok(PackageManager::Dpkg),
        "rpm"  => Ok(PackageManager::Rpm),
        "test" => Ok(PackageManager::Test),
        s      => Err(s.to_string()),
    }
}

impl Decodable for PackageManager {

    fn decode<D: Decoder>(d: &mut D) -> Result<PackageManager, D::Error> {
        d.read_str().and_then(|s| parse_package_manager(s)
                              .map_err(|e| d.error(&e)))
    }
}
