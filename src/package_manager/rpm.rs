use std::process::Command;

use datatype::{Error, Package, UpdateResultCode};
use package_manager::dpkg::parse_package; // XXX: Move somewhere better?


pub fn installed_packages() -> Result<Vec<Package>, Error> {
    Command::new("rpm").arg("-qa").arg("--queryformat").arg("%{NAME} %{SIZE}\n")
        .output()
        .map_err(|e| Error::PackageError(format!("Error fetching packages: {}", e)))
        .and_then(|c| {
            String::from_utf8(c.stdout)
                .map_err(|e| Error::ParseError(format!("Error parsing package: {}", e)))
                .map(|s| s.lines().map(|n| String::from(n)).collect::<Vec<String>>())
        })
        .and_then(|lines| {
            lines.iter()
                .map(|line| parse_package(line))
                .collect::<Result<Vec<Package>, _>>()
        })
}

pub fn install_package(path: &str) -> Result<(UpdateResultCode, String), (UpdateResultCode, String)> {
    let output = try!(Command::new("rpm").arg("-ivh").arg(path)
                      .output()
                      .map_err(|e| {
                          (UpdateResultCode::GENERAL_ERROR, format!("{:?}", e))
                      }));

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    match output.status.code() {
        Some(0) => Ok((UpdateResultCode::OK, stdout)),
        _ => if (&stderr).contains("already installed") {
            Ok((UpdateResultCode::ALREADY_PROCESSED, stderr))
        } else {
            Err((UpdateResultCode::INSTALL_FAILED, stderr))
        }
    }
}
