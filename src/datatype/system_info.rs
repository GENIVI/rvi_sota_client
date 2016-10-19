use rustc_serialize::{Decoder, Decodable};
use std::process::Command;

use datatype::Error;


/// A reference to the command that will report on the system information.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SystemInfo {
    command: String
}

impl SystemInfo {
    /// Instantiate a new type to report on the system information.
    pub fn new(command: &str) -> Option<SystemInfo> {
        if command == "" {
            None
        } else {
            Some(SystemInfo { command: command.to_string() })
        }
    }

    /// Generate a new report of the system information.
    pub fn report(&self) -> Result<String, Error> {
        Command::new(&self.command)
            .output().map_err(|err| Error::SystemInfo(err.to_string()))
            .and_then(|info| String::from_utf8(info.stdout).map_err(Error::FromUtf8))
    }
}

impl Default for SystemInfo {
    fn default() -> SystemInfo {
        SystemInfo::new("./system_info.sh").expect("couldn't build command")
    }
}

impl Decodable for SystemInfo {
    fn decode<D: Decoder>(d: &mut D) -> Result<SystemInfo, D::Error> {
        d.read_str().and_then(|s| SystemInfo::new(&s).ok_or(d.error("bad SystemInfo command path")))
    }
}
