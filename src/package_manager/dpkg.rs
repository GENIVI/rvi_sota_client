use std::process::Command;

use datatype::Error;
use datatype::Package;
use package_manager;


#[allow(dead_code)]
pub struct Dpkg { a: u16 } // remove dummy field once braced_empty_structs feature is in stable

impl Dpkg {
    pub fn new() -> Dpkg {
        Dpkg { a: 0 }
    }
}

impl package_manager::PackageManager for Dpkg {

    fn new() -> Dpkg {
        return Dpkg::new();
    }

    fn installed_packages(&self) -> Result<Vec<Package>, Error> {
        Command::new("dpkg-query").arg("-f").arg("${Package} ${Version}\n").arg("-W")
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
}

pub fn parse_package(line: &str) -> Result<Package, Error> {
    match line.splitn(2, ' ').collect::<Vec<_>>() {
        ref parts if parts.len() == 2 => Ok(Package { name: String::from(parts[0]),
                                                      version: String::from(parts[1]) }),
        _ => Err(Error::ParseError(format!("Couldn't parse package: {}", line)))
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
                   "Couldn't parse package: foobar".to_string());
    }

}
