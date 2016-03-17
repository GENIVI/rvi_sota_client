use std::fmt::{Display, Formatter, Result as FmtResult};

pub type Version = String;

#[derive(Debug, PartialEq, Eq, RustcEncodable, RustcDecodable)]
pub struct Package {
    pub name: String,
    pub version: Version
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", self.name, self.version)
    }
}
