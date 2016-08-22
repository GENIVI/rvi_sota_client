use rustc_serialize::{Decoder, Decodable};
use std::env::temp_dir;
use std::str::FromStr;

use datatype::{Error, Package, UpdateResultCode};
use package_manager::{deb, otb, rpm, tpm};
use tempfile::NamedTempFile;


pub type InstallOutcome = (UpdateResultCode, String);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PackageManager {
    Deb,
    Rpm,
    File { filename: String, succeeds: bool },
    OstreeBasic { repodir: String }
}

impl PackageManager {
    pub fn new_file(succeeds: bool) -> Self {
        PackageManager::File {
            filename: NamedTempFile::new_in(temp_dir()).expect("couldn't create temporary file")
                .path().file_name().expect("couldn't get file name")
                .to_str().expect("couldn't parse file name").to_string(),
            succeeds: succeeds
        }
    }

    pub fn installed_packages(&self) -> Result<Vec<Package>, Error> {
        match *self {
            PackageManager::Deb => deb::installed_packages(),
            PackageManager::Rpm => rpm::installed_packages(),
            PackageManager::File { ref filename, .. } => tpm::installed_packages(filename),
            PackageManager::OstreeBasic { ref repodir } => otb::installed_packages(repodir),
        }
    }

    pub fn install_package(&self, path: &str) -> Result<InstallOutcome, InstallOutcome> {
        match *self {
            PackageManager::Deb => deb::install_package(path),
            PackageManager::Rpm => rpm::install_package(path),
            PackageManager::File { ref filename, succeeds } => tpm::install_package(filename, path, succeeds),
            PackageManager::OstreeBasic { ref repodir } => otb::install_package(repodir, path),
        }
    }

    pub fn extension(&self) -> String {
        match *self {
            PackageManager::Deb => "deb".to_string(),
            PackageManager::Rpm => "rpm".to_string(),
            PackageManager::File { ref filename, .. } => filename.to_string(),
            PackageManager::OstreeBasic {..} => "otb".to_string(),
        }
    }
}

impl FromStr for PackageManager {
    type Err = Error;

    fn from_str(s: &str) -> Result<PackageManager, Error> {
        match s.to_lowercase().as_str() {
            "deb" => Ok(PackageManager::Deb),
            "rpm" => Ok(PackageManager::Rpm),

            file if file.len() > 5 && file[..5].as_bytes() == b"file:" => {
                Ok(PackageManager::File { filename: file[5..].to_string(), succeeds: true })
            },

            repo if repo.len() > 4 && repo[..4].as_bytes() == b"otb:" => {
                Ok(PackageManager::OstreeBasic { repodir: repo[4..].to_string() })
            }

            _ => Err(Error::Parse(format!("unknown package manager: {}", s)))
        }
    }
}

impl Decodable for PackageManager {
    fn decode<D: Decoder>(d: &mut D) -> Result<PackageManager, D::Error> {
        d.read_str().and_then(|s| Ok(s.parse::<PackageManager>().expect("couldn't parse PackageManager")))
    }
}

pub fn parse_package(line: &str) -> Result<Package, Error> {
    match line.splitn(2, ' ').collect::<Vec<_>>() {
        ref parts if parts.len() == 2 => {
            // HACK: strip left single quotes from stdout
            Ok(Package {
                name:    String::from(parts[0].trim_left_matches('\'')),
                version: String::from(parts[1])
            })
        },
        _ => Err(Error::Parse(format!("Couldn't parse package: {}", line)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use datatype::Package;


    #[test]
    fn test_parses_normal_package() {
        assert_eq!(parse_package("uuid-runtime 2.20.1-5.1ubuntu20.7").unwrap(),
                   Package {
                       name: "uuid-runtime".to_string(),
                       version: "2.20.1-5.1ubuntu20.7".to_string()
                   });
    }

    #[test]
    fn test_separates_name_and_version_correctly() {
        assert_eq!(parse_package("vim 2.1 foobar").unwrap(),
                   Package {
                       name: "vim".to_string(),
                       version: "2.1 foobar".to_string()
                   });
    }

    #[test]
    fn test_rejects_bogus_input() {
        assert_eq!(format!("{}", parse_package("foobar").unwrap_err()),
                   "Parse error: Couldn't parse package: foobar".to_string());
    }
}
