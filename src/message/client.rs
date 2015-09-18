use std::vec::Vec;
use super::server::PackageId;
use super::server::BackendServices;

#[derive(RustcDecodable, Clone, PartialEq, Eq, Debug)]
pub struct UserPackage {
    pub package: PackageId,
    pub size: u64
}

pub struct UserMessage {
    pub packages: Vec<UserPackage>,
    pub services: BackendServices
}
