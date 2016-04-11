use std::process::Command;
use datatype::Error;
use datatype::OtaConfig;
use datatype::Package;
use datatype::UpdateResultCode;
use package_manager::PackageManager;
use package_manager::dpkg::parse_package as parse_package;

pub struct Rpm;

pub static RPM: &'static PackageManager = &Rpm;

impl PackageManager for Rpm {
    fn installed_packages(&self, _: &OtaConfig) -> Result<Vec<Package>, Error> {
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

    fn install_package(&self, _: &OtaConfig, path: &str) -> Result<(UpdateResultCode, String), (UpdateResultCode, String)> {
        let output = try!(Command::new("rpm").arg("-ivh").arg(path)
                          .output()
                          .map_err(|e| {
                              (UpdateResultCode::GENERAL_ERROR, format!("{:?}", e))
                          }));

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

        match output.status.code() {
            Some(0) => Ok((UpdateResultCode::OK, stdout)),
            _ => if (&stdout).contains("already installed") {
                Ok((UpdateResultCode::ALREADY_PROCESSED, stdout))
            } else {
                Err((UpdateResultCode::INSTALL_FAILED, stdout))
            }
        }
    }
}
