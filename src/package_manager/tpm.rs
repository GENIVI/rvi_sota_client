use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::iter::Iterator;

use datatype::Error;
use datatype::OtaConfig;
use datatype::Package;
use datatype::UpdateResultCode;
use package_manager::PackageManager;


// The test package manager.
pub struct Tpm;

pub static TPM: &'static PackageManager = &Tpm;

impl PackageManager for Tpm {

    fn installed_packages(&self, config: &OtaConfig) -> Result<Vec<Package>, Error> {

        let f        = try!(File::open(config.packages_dir.clone() +
                                       &config.package_manager.extension()));
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

    fn install_package(&self, config: &OtaConfig, pkg: &str) -> Result<(UpdateResultCode, String), (UpdateResultCode, String)> {
        fn install(config: &OtaConfig, pkg: &str) -> Result<(), Error> {
            let f = try!(OpenOptions::new()
                         .create(true)
                         .write(true)
                         .append(true)
                         .open(config.packages_dir.clone() +
                               &config.package_manager.extension()));

            let mut writer = BufWriter::new(f);

            try!(writer.write(pkg.as_bytes()));
            try!(writer.write(b"\n"));

            return Ok(())
        }

        match install(&config, &pkg) {
            Ok(_) => Ok((UpdateResultCode::OK, "".to_string())),
            Err(e) => Err((UpdateResultCode::INSTALL_FAILED, format!("{:?}", e)))
        }
    }

}


#[cfg(test)]
mod tests {

    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;

    use super::*;
    use datatype::OtaConfig;
    use datatype::Package;
    use datatype::PackageManager;
    use package_manager::PackageManager as PackageManagerTrait;

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

    fn make_config(file: &str) -> OtaConfig {

        let packages_dir    = "/tmp/".to_string();
        let package_manager = PackageManager::File(file.to_string());

        let mut config = OtaConfig::default();

        config = OtaConfig {
            packages_dir:    packages_dir,
            package_manager: package_manager,
            .. config
        };

        return config

    }

    #[test]
    fn test_installed_packages() {

        let config = make_config("test1");

        let mut f = File::create(config.packages_dir.clone() +
                                 &config.package_manager.extension()).unwrap();

        f.write(b"apa 0.0.0\n").unwrap();
        f.write(b"bepa 1.0.0").unwrap();

        assert_eq!(Tpm.installed_packages(&config).unwrap(), vec!(pkg1(), pkg2()));

    }

    #[test]
    fn bad_installed_packages() {

        let config = make_config("test2");

        let mut f = File::create(config.packages_dir.clone() +
                                 &config.package_manager.extension()).unwrap();

        f.write(b"cepa-2.0.0\n").unwrap();

        assert_eq!(Tpm.installed_packages(&config).unwrap(), Vec::new());

    }

    #[test]
    fn test_install_package() {

        let config = make_config("test3");

        let _ = fs::remove_file(config.packages_dir.to_string() +
                                &config.package_manager.extension());

        Tpm.install_package(&config, "apa 0.0.0").unwrap();
        Tpm.install_package(&config, "bepa 1.0.0").unwrap();

        assert_eq!(Tpm.installed_packages(&config).unwrap(), vec!(pkg1(), pkg2()));

    }

}
