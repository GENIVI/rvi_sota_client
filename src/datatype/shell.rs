use std::process::Command;

use datatype::Error;


/// Generate a new system information report.
pub fn system_info(cmd: &str) -> Result<String, Error> {
    Command::new(cmd)
        .output()
        .map_err(|err| Error::SystemInfo(err.to_string()))
        .and_then(|info| String::from_utf8(info.stdout).map_err(Error::FromUtf8))
}
