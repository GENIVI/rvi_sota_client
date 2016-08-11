use rustc_serialize::{Decoder, Decodable};
use rustc_serialize::json::Json;

use std::process::Command;
use std::str::FromStr;

use datatype::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SystemInfo {
    script : String
}

impl SystemInfo {

    pub fn new(script: String) -> SystemInfo {
        SystemInfo { script: script}
    }

    pub fn get_json(&self) -> Result<Json,Error> {
        Command::new(&self.script)
            .output().map_err(|e| Error::SystemInfo(format!("Error getting system info: {}", e)))
            .and_then(|c| String::from_utf8(c.stdout)
                     .map_err(|e| Error::FromUtf8(e)))
            .and_then(|s| Json::from_str(&s)
                     .map_err(|e| Error::JsonParser(e)))
    }
}

impl Default for SystemInfo {
    fn default() -> SystemInfo {
        SystemInfo::new("sota-system-info".to_string())
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
        d.read_str().and_then(|s| s.parse::<SystemInfo>()
                    .map_err(|e|d.error(&format!("Error couldn't parse SystemInfo: {}",e))))
    }
}
