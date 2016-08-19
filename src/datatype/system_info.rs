use rustc_serialize::{Decoder, Decodable};
use rustc_serialize::json::Json;
use std::process::Command;
use std::str::FromStr;

use datatype::Error;


/// A reference to the command that will report on the system information.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SystemInfo {
    command: String
}

impl SystemInfo {
    /// Instantiate a new type to report on the system information.
    pub fn new(command: String) -> SystemInfo {
        SystemInfo { command: command }
    }

    /// Generate a new report of the system information.
    pub fn report(&self) -> Result<Json, Error> {
        Command::new(&self.command)
            .output().map_err(|err| Error::SystemInfo(err.to_string()))
            .and_then(|info| String::from_utf8(info.stdout).map_err(Error::FromUtf8))
            .and_then(|text| Json::from_str(&text).map_err(Error::JsonParser))
    }
}

impl Default for SystemInfo {
    fn default() -> SystemInfo {
        SystemInfo::new("system_info.sh".to_string())
    }
}

impl FromStr for SystemInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<SystemInfo, Error> {
        Ok(SystemInfo::new(s.to_string()))
    }
}

impl Decodable for SystemInfo {
    fn decode<D: Decoder>(d: &mut D) -> Result<SystemInfo, D::Error> {
        d.read_str().and_then(|s| Ok(s.parse::<SystemInfo>().unwrap()))
    }
}
