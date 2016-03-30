use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::iter::Iterator;

use datatype::Error;
use datatype::Package;
use package_manager::PackageManager;


// The test package manager.
pub struct Tpm;

pub static TPM: &'static PackageManager = &Tpm;

static PATH: &'static str  = "/tmp/packages.tpm";

impl PackageManager for Tpm {

    fn installed_packages(&self) -> Result<Vec<Package>, Error> {

        let f        = try!(File::open(PATH));
        let reader   = BufReader::new(f);
        let mut pkgs = Vec::new();

        for line in reader.lines() {

            let line  = try!(line);
            let parts = line.split(' ');

            if parts.clone().count() == 2 {
                if let Some(name) = parts.clone().nth(0) {
                    if let Some(version) = parts.clone().nth(1) {
                        pkgs.push(Package {
                            name:    name.to_string(),
                            version: version.to_string()
                        });
                    }
                }
            }

        }

        return Ok(pkgs)

    }

    fn install_package(&self, pkg: &str) -> Result<(), Error> {

        let f = try!(OpenOptions::new()
                     .create(true)
                     .append(true)
                     .open(PATH));

        let mut writer = BufWriter::new(f);

        try!(writer.write(pkg.as_bytes()));
        try!(writer.write(b"\n"));

        return Ok(())

    }

}


#[cfg(test)]
mod tests {

    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;

    use super::*;
    use datatype::Package;
    use package_manager::PackageManager;

    fn pkg1() -> Package {
        Package {
            name:    "apa".to_string(),
            version: "0.0.0".to_string()
        }
    }

    fn pkg2() -> Package {
        Package {
            name:    "bepa".to_string(),
            version: "1.0.0".to_string()
        }
    }

    #[test]
    fn test_installed_packages() {

        let _     = fs::remove_file(super::PATH);
        let mut f = File::create(super::PATH).unwrap();

        f.write(b"apa 0.0.0\n").unwrap();
        f.write(b"bepa 1.0.0").unwrap();

        assert_eq!(Tpm.installed_packages().unwrap(), vec!(pkg1(), pkg2()));

    }

    #[test]
    fn bad_installed_packages() {

        let _     = fs::remove_file(super::PATH);
        let mut f = File::create(super::PATH).unwrap();
        f.write(b"cepa-2.0.0\n").unwrap();

        assert_eq!(Tpm.installed_packages().unwrap(), Vec::new());

    }

    #[test]
    fn test_install_package() {

        let _ = fs::remove_file(super::PATH);
        Tpm.install_package("apa 0.0.0").unwrap();
        Tpm.install_package("bepa 1.0.0").unwrap();

        assert_eq!(Tpm.installed_packages().unwrap(), vec!(pkg1(), pkg2()));

    }

}
